pub mod cursor;
pub mod gutter;
pub mod keymap;
pub mod statusline;

use std::collections::HashMap;

use cursor::{LuaCursorConfig, LuaCursorModeStyle, LuaCursorStyle};
use glyph_core::config::{
    Config, CursorConfig, CursorModeStyle, CursorStyle, GutterAnchor, GutterConfig, KeymapConfig, KeymapOptions,
    LineNumbersConfig, MappableCommandConfig, SignColumnConfig, StatuslineConfig, StatuslineContent, StatuslineSection,
    StatuslineStyle, UserCommand,
};
use glyph_core::dirs::DIRS;
use glyph_core::editor::Mode;
use glyph_core::highlights::HighlightGroup;
use gutter::{LuaGutterAnchor, LuaGutterConfig, LuaLineNumbersConfig, LuaSignColumnConfig};
use keymap::{LuaKeymapConfig, LuaKeymapOptions, LuaMappableCommand};
use mlua::{FromLua, LuaSerdeExt};
use statusline::{LuaStatuslineConfig, LuaStatuslineContent, LuaStatuslineSection, LuaStatuslineStyle};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::RuntimeMessage;

#[derive(Debug)]
pub struct LuaConfig<'cfg> {
    pub cursor: LuaCursorConfig,
    pub gutter: LuaGutterConfig,
    pub keymaps: HashMap<Mode, Vec<LuaKeymapConfig<'cfg>>>,
    pub highlight_groups: HashMap<String, HighlightGroup>,
    pub statusline: LuaStatuslineConfig,
    pub user_commands: HashMap<String, mlua::Function>,
    pub scroll_offset: usize,
}

pub fn load<'cfg>(
    runtime: &mlua::Lua,
    runtime_receiver: &mut UnboundedReceiver<RuntimeMessage<'static>>,
) -> crate::error::Result<LuaConfig<'cfg>> {
    let config = DIRS.get().expect("failed to get dirs").config();
    let init = config.join("init.lua");

    if let Ok(content) = std::fs::read_to_string(&init) {
        runtime
            .load(content)
            .set_name(init.to_string_lossy())
            .eval::<mlua::Value>()
            .expect("error in lua ----- {err:?}");
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

    let glyph = crate::get_or_create_module(runtime, "glyph")?;
    let config = glyph.get::<mlua::Table>("options")?;

    let scroll_offset = runtime.from_value::<usize>(config.get::<mlua::Value>("scroll_offset")?)?;
    let gutter = runtime.from_value::<LuaGutterConfig>(config.get::<mlua::Value>("gutter")?)?;

    let cursor = LuaCursorConfig::from_lua(config.get::<mlua::Value>("cursor")?, runtime)?;
    let statusline = LuaStatuslineConfig::from_lua(config.get::<mlua::Value>("statusline")?, runtime)?;

    Ok(LuaConfig {
        cursor,
        gutter,
        keymaps,
        statusline,
        user_commands,
        scroll_offset,
        highlight_groups,
    })
}

struct SetupMessagesResult<'cfg> {
    highlight_groups: HashMap<String, HighlightGroup>,
    keymaps: HashMap<Mode, Vec<LuaKeymapConfig<'cfg>>>,
    user_commands: HashMap<String, mlua::Function>,
}

fn handle_setup_messages(messages: Vec<RuntimeMessage<'_>>) -> SetupMessagesResult<'_> {
    let mut highlight_groups = HashMap::default();

    let mut keymaps = HashMap::default();
    keymaps.insert(Mode::Normal, Vec::default());
    keymaps.insert(Mode::Insert, Vec::default());
    keymaps.insert(Mode::Command, Vec::default());

    let mut user_commands = HashMap::default();

    for message in messages {
        match message {
            RuntimeMessage::Error(error) => panic!("{error:?}"),
            RuntimeMessage::UpdateHighlightGroup(name, group) => _ = highlight_groups.insert(name, group),
            RuntimeMessage::UserCommandCreate(name, callback) => _ = user_commands.insert(name, callback),
            RuntimeMessage::SetKeymap(lua_keymap) => {
                let mode_maps = keymaps.get_mut(&lua_keymap.mode).expect("should have a mode keymaps");
                mode_maps.push(lua_keymap);
            }
            // Some runtime messages are ignored here as we don't want to handle them within
            // configuration context.
            //
            // Only messages that change configuration values should be considered when loading
            // configuration.
            RuntimeMessage::Quit(_) => {}
            RuntimeMessage::OpenFile(_) => {}
            RuntimeMessage::Write(_) => {}
        };
    }

    SetupMessagesResult {
        highlight_groups,
        keymaps,
        user_commands,
    }
}

pub struct UserCommandCallback(mlua::Function);

impl UserCommand for UserCommandCallback {
    fn call(&self) -> Result<(), String> {
        self.0.call::<()>(()).unwrap();
        Ok(())
    }
}

impl<'a> From<LuaConfig<'a>> for Config<'a> {
    fn from(value: LuaConfig<'a>) -> Self {
        Self {
            cursor: value.cursor.into(),
            gutter: value.gutter.into(),
            scroll_offset: value.scroll_offset,
            user_commands: value
                .user_commands
                .into_iter()
                .map(|(k, cb)| (k, Box::new(UserCommandCallback(cb)) as Box<dyn UserCommand>))
                .collect(),
            statusline: value.statusline.into(),
            highlight_groups: value.highlight_groups,
            keymaps: value
                .keymaps
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().map(Into::into).into()))
                .collect(),
        }
    }
}

impl From<LuaCursorConfig> for CursorConfig {
    fn from(value: LuaCursorConfig) -> Self {
        Self {
            style: value.style.into(),
        }
    }
}

impl From<LuaCursorModeStyle> for CursorModeStyle {
    fn from(value: LuaCursorModeStyle) -> Self {
        Self {
            normal: value.normal.into(),
            insert: value.insert.into(),
            command: value.command.into(),
            visual: value.visual.into(),
        }
    }
}

impl From<LuaCursorStyle> for CursorStyle {
    fn from(value: LuaCursorStyle) -> Self {
        match value {
            LuaCursorStyle::Block => CursorStyle::Block,
            LuaCursorStyle::SteadyBar => CursorStyle::SteadyBar,
        }
    }
}

impl From<LuaGutterConfig> for GutterConfig {
    fn from(value: LuaGutterConfig) -> Self {
        Self {
            enabled: value.enabled,
            anchor: value.anchor.into(),
            line_numbers: value.line_numbers.into(),
            sign_column: value.sign_column.into(),
        }
    }
}

impl From<LuaGutterAnchor> for GutterAnchor {
    fn from(value: LuaGutterAnchor) -> Self {
        match value {
            LuaGutterAnchor::Left => GutterAnchor::Left,
            LuaGutterAnchor::Right => GutterAnchor::Right,
        }
    }
}

impl From<LuaLineNumbersConfig> for LineNumbersConfig {
    fn from(value: LuaLineNumbersConfig) -> Self {
        match value {
            LuaLineNumbersConfig::Absolute => LineNumbersConfig::Absolute,
            LuaLineNumbersConfig::Relative => LineNumbersConfig::Relative,
            LuaLineNumbersConfig::RelativeNumbered => LineNumbersConfig::RelativeNumbered,
        }
    }
}

impl From<LuaSignColumnConfig> for SignColumnConfig {
    fn from(value: LuaSignColumnConfig) -> Self {
        match value {
            LuaSignColumnConfig::All => SignColumnConfig::All,
            LuaSignColumnConfig::None => SignColumnConfig::None,
        }
    }
}

impl From<LuaStatuslineConfig> for StatuslineConfig {
    fn from(value: LuaStatuslineConfig) -> Self {
        Self {
            left: value.left.into_iter().map(|s| s.into()).collect(),
            right: value.right.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl From<LuaStatuslineSection> for StatuslineSection {
    fn from(value: LuaStatuslineSection) -> Self {
        Self {
            content: value.content.into(),
            style: value.style.into(),
        }
    }
}

impl From<LuaStatuslineContent> for StatuslineContent {
    fn from(value: LuaStatuslineContent) -> Self {
        match value {
            LuaStatuslineContent::Immediate(value) => StatuslineContent::Immediate(value),
            LuaStatuslineContent::Dynamic(function) => StatuslineContent::Dynamic(Box::new(move || {
                function
                    .call::<mlua::String>(())
                    .map(|s| s.to_string_lossy())
                    .unwrap_or_default()
            })),
        }
    }
}

impl From<LuaStatuslineStyle> for StatuslineStyle {
    fn from(value: LuaStatuslineStyle) -> Self {
        match value {
            LuaStatuslineStyle::HighlightGroup(highlight_group) => StatuslineStyle::HighlightGroup(highlight_group),
            LuaStatuslineStyle::Named(name) => StatuslineStyle::Named(name),
        }
    }
}

impl<'a> From<LuaKeymapConfig<'a>> for (String, KeymapConfig<'a>) {
    fn from(value: LuaKeymapConfig<'a>) -> Self {
        (
            value.keys,
            KeymapConfig {
                mode: value.mode,
                command: value.command.into(),
                options: value.options.into(),
            },
        )
    }
}

impl<'a> From<LuaMappableCommand<'a>> for MappableCommandConfig<'a> {
    fn from(value: LuaMappableCommand<'a>) -> Self {
        match value {
            LuaMappableCommand::Borrowed(command) => MappableCommandConfig::Borrowed(command),
            LuaMappableCommand::Owned(command) => MappableCommandConfig::Owned(command),
        }
    }
}

impl From<LuaKeymapOptions> for KeymapOptions {
    fn from(value: LuaKeymapOptions) -> Self {
        Self {
            description: value.description,
        }
    }
}
