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
    SteadyBar,
}

impl<S: AsRef<str>> From<S> for CursorStyle {
    fn from(value: S) -> CursorStyle {
        match value.as_ref() {
            "block" => CursorStyle::Block,
            "steady_bar" => CursorStyle::SteadyBar,
            _ => CursorStyle::Block,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct CursorConfig {
    pub style: CursorModeStyle,
}

#[derive(Debug, Default, Clone)]
pub struct CursorModeStyle {
    pub normal: CursorStyle,
    pub insert: CursorStyle,
    pub command: CursorStyle,
    pub visual: CursorStyle,
}

impl CursorModeStyle {
    pub fn block() -> CursorModeStyle {
        CursorModeStyle {
            normal: CursorStyle::Block,
            insert: CursorStyle::Block,
            command: CursorStyle::Block,
            visual: CursorStyle::Block,
        }
    }

    pub fn bar() -> CursorModeStyle {
        CursorModeStyle {
            normal: CursorStyle::SteadyBar,
            insert: CursorStyle::SteadyBar,
            command: CursorStyle::SteadyBar,
            visual: CursorStyle::SteadyBar,
        }
    }
}

impl<S: AsRef<str>> From<S> for CursorModeStyle {
    fn from(value: S) -> Self {
        match value.as_ref() {
            "block" => CursorModeStyle::block(),
            "steady_bar" => CursorModeStyle::bar(),
            _ => CursorModeStyle::block(),
        }
    }
}

impl FromLua for CursorConfig {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        let Value::Table(cursor) = value else {
            return Err(mlua::Error::runtime("cursor must be a table"));
        };

        let style = cursor.get::<Value>("style")?;
        Ok(match style {
            Value::String(style) => CursorConfig {
                style: CursorModeStyle::from(style.to_string_lossy()),
            },
            Value::Table(table) => CursorConfig {
                style: CursorModeStyle {
                    normal: table.get::<String>("normal").unwrap_or("block".into()).into(),
                    insert: table.get::<String>("insert").unwrap_or("block".into()).into(),
                    command: table.get::<String>("command").unwrap_or("block".into()).into(),
                    visual: table.get::<String>("visual").unwrap_or("block".into()).into(),
                },
            },
            _ => return Err(mlua::Error::runtime("cursor style must be a string or table")),
        })
    }
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

        //let scroll_offset = runtime.from_value::<usize>(config.get::<Value>("scroll_offset")?)?;
        let gutter = runtime.from_value::<GutterConfig>(config.get::<Value>("gutter")?)?;

        let cursor = CursorConfig::from_lua(config.get::<Value>("cursor")?, runtime)?;
        let statusline = StatuslineConfig::from_lua(config.get::<Value>("statusline")?, runtime)?;

        Ok(Config {
            cursor,
            gutter,
            keymaps,
            statusline,
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
