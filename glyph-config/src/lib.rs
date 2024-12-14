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
use mlua::{FromLua, Function, Lua, LuaSerdeExt, Table, Value};
use serde::Deserialize;
use tokio::sync::mpsc::UnboundedReceiver;

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
    scroll_offset: usize,
    pub user_commands: HashMap<String, Function>,
    pub statusline: StatuslineConfig,
    pub highlight_groups: HashMap<String, HighlightGroup>,
    pub keymaps: HashMap<Mode, Trie<KeymapConfig<'cfg>>>,
}

impl<'cfg> Config<'cfg> {
    pub fn load(
        runtime: &Lua,
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

        let SetupMessagesResult {
            highlight_groups,
            keymaps,
            user_commands,
        } = handle_setup_messages(setup_messages);

        let glyph = glyph_runtime::get_or_create_module(runtime, "glyph")?;
        let config = glyph.get::<Table>("options")?;

        let scroll_offset = runtime.from_value::<usize>(config.get::<Value>("scroll_offset")?)?;
        let cursor = runtime.from_value::<CursorConfig>(config.get::<Value>("cursor")?)?;
        let gutter = runtime.from_value::<GutterConfig>(config.get::<Value>("gutter")?)?;
        let statusline = StatuslineConfig::from_lua(config.get::<Value>("statusline")?, runtime)?;

        Ok(Config {
            cursor,
            gutter,
            keymaps,
            statusline,
            scroll_offset,
            user_commands,
            highlight_groups,
        })
    }

    pub fn cursor(&self) -> &CursorConfig {
        &self.cursor
    }

    pub fn gutter(&self) -> &GutterConfig {
        &self.gutter
    }
}

struct SetupMessagesResult<'cfg> {
    highlight_groups: HashMap<String, HighlightGroup>,
    keymaps: HashMap<Mode, Trie<KeymapConfig<'cfg>>>,
    user_commands: HashMap<String, Function>,
}

fn handle_setup_messages(messages: Vec<RuntimeMessage>) -> SetupMessagesResult {
    let mut highlight_groups = HashMap::default();

    let mut keymaps = HashMap::default();
    keymaps.insert(Mode::Normal, Trie::default());
    keymaps.insert(Mode::Insert, Trie::default());
    keymaps.insert(Mode::Command, Trie::default());

    let mut user_commands = HashMap::default();

    for message in messages {
        match message {
            RuntimeMessage::Error(error) => println!("{error:?}"),
            RuntimeMessage::UpdateHighlightGroup(name, group) => _ = highlight_groups.insert(name, group),
            RuntimeMessage::UserCommandCreate(name, callback) => _ = user_commands.insert(name, callback),
            RuntimeMessage::SetKeymap(lua_keymap) => {
                let keymap = KeymapConfig {
                    mode: lua_keymap.mode,
                    command: lua_keymap.command.into(),
                    options: lua_keymap.options.into(),
                };
                let mode_maps = keymaps.get_mut(&lua_keymap.mode).unwrap();
                mode_maps.add_word(&lua_keymap.keys, keymap);
            }
            RuntimeMessage::Quit(_) => {}
            RuntimeMessage::Write(_) => {}
        };
    }

    SetupMessagesResult {
        highlight_groups,
        keymaps,
        user_commands,
    }
}
