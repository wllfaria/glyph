pub mod dirs;

use std::collections::HashMap;
use std::fmt::Debug;

use dirs::DIRS;
use glyph_core::command::{Context, MappableCommand};
use glyph_core::editor::Mode;
use glyph_core::highlights::HighlightGroup;
use glyph_runtime::keymap::{LuaKeymapOpts, LuaMappableCommand};
use glyph_runtime::statusline::StatuslineConfig;
use glyph_runtime::RuntimeMessage;
use glyph_trie::Trie;
use mlua::{Lua, LuaSerdeExt, Table, Value};
use serde::Deserialize;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub type GlyphConfig<'a> = &'a Config<'a>;

fn yes() -> bool {
    true
}

#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    #[default]
    Block,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CursorConfig {
    #[serde(default)]
    pub style: CursorStyle,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GutterAnchor {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineNumbersConfig {
    #[default]
    Absolute,
    Relative,
    #[serde(rename = "relative_numbered")]
    RelativeNumbered,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SignColumnConfig {
    #[default]
    All,
    None,
}

impl SignColumnConfig {
    pub fn size(&self) -> u16 {
        match self {
            SignColumnConfig::None => 0,
            _ => 1,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct GutterConfig {
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub anchor: GutterAnchor,
    #[serde(default)]
    pub line_numbers: LineNumbersConfig,
    #[serde(default)]
    pub sign_column: SignColumnConfig,
}

#[derive(Debug)]
pub struct KeymapOptions {
    pub description: String,
}

impl From<LuaKeymapOpts> for KeymapOptions {
    fn from(opts: LuaKeymapOpts) -> KeymapOptions {
        KeymapOptions {
            description: opts.description,
        }
    }
}

#[derive(Debug)]
pub enum MappableCommandConfig<'cmd> {
    Borrowed(&'cmd MappableCommand),
    Owned(MappableCommand),
}

impl MappableCommandConfig<'_> {
    pub fn run(&self, ctx: &mut Context) {
        match self {
            MappableCommandConfig::Borrowed(MappableCommand::Static { fun, .. }) => fun(ctx),
            MappableCommandConfig::Borrowed(MappableCommand::Dynamic { callback, .. }) => callback(),
            MappableCommandConfig::Owned(MappableCommand::Static { fun, .. }) => fun(ctx),
            MappableCommandConfig::Owned(MappableCommand::Dynamic { callback, .. }) => callback(),
        }
    }
}

impl<'a> From<LuaMappableCommand<'a>> for MappableCommandConfig<'a> {
    fn from(cmd: LuaMappableCommand<'a>) -> MappableCommandConfig<'a> {
        match cmd {
            LuaMappableCommand::Borrowed(inner) => MappableCommandConfig::Borrowed(inner),
            LuaMappableCommand::Owned(inner) => MappableCommandConfig::Owned(inner),
        }
    }
}

#[derive(Debug)]
pub struct KeymapConfig<'cfg> {
    pub mode: Mode,
    pub command: MappableCommandConfig<'cfg>,
    pub options: KeymapOptions,
}

#[derive(Debug)]
pub struct Config<'cfg> {
    cursor: CursorConfig,
    gutter: GutterConfig,
    pub statusline: StatuslineConfig,
    pub highlight_groups: HashMap<String, HighlightGroup>,
    pub keymaps: Trie<KeymapConfig<'cfg>>,
}

impl<'cfg> Config<'cfg> {
    pub fn load(
        runtime: &Lua,
        _runtime_sender: UnboundedSender<RuntimeMessage<'static>>,
        runtime_receiver: &mut UnboundedReceiver<RuntimeMessage<'static>>,
    ) -> glyph_runtime::error::Result<Config<'cfg>> {
        let config = DIRS.get().unwrap().config();
        let init = config.join("init.lua");

        if let Ok(content) = std::fs::read_to_string(&init) {
            if let Err(err) = runtime.load(content).set_name(init.to_string_lossy()).eval::<Value>() {
                todo!("error in lua ----- {err:?}");
            }
        }

        let mut setup_messages = vec![];
        while let Ok(message) = runtime_receiver.try_recv() {
            setup_messages.push(message);
        }

        let (highlight_groups, keymaps) = handle_setup_messages(setup_messages);

        let glyph = glyph_runtime::get_or_create_module(runtime, "glyph")?;
        let config = glyph.get::<Table>("options")?;

        let cursor = runtime.from_value::<CursorConfig>(config.get::<Value>("cursor")?)?;
        let gutter = runtime.from_value::<GutterConfig>(config.get::<Value>("gutter")?)?;

        Ok(Config {
            cursor,
            gutter,
            highlight_groups,
            keymaps,
            // will be loaded when the editors starts up
            statusline: Default::default(),
        })
    }

    pub fn cursor(&self) -> &CursorConfig {
        &self.cursor
    }

    pub fn gutter(&self) -> &GutterConfig {
        &self.gutter
    }
}

fn handle_setup_messages(messages: Vec<RuntimeMessage>) -> (HashMap<String, HighlightGroup>, Trie<KeymapConfig>) {
    let mut highlight_groups = HashMap::default();
    let mut keymaps = Trie::default();

    for message in messages {
        match message {
            RuntimeMessage::Error(error) => println!("{error:?}"),
            RuntimeMessage::UpdateHighlightGroup(name, group) => _ = highlight_groups.insert(name, group),
            RuntimeMessage::SetKeymap(lua_keymap) => {
                let keymap = KeymapConfig {
                    mode: lua_keymap.mode,
                    command: lua_keymap.command.into(),
                    options: lua_keymap.options.into(),
                };
                keymaps.add_word(&lua_keymap.keys, keymap);
            }
        };
    }

    (highlight_groups, keymaps)
}
