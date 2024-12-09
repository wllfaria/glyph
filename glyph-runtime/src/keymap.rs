use glyph_core::command::MappableCommand;
use glyph_core::editor::Mode;
use mlua::{Lua, LuaSerdeExt, Table, Value};
use serde::Deserialize;
use tokio::sync::mpsc::UnboundedSender;

use crate::error::Result;
use crate::RuntimeMessage;

#[derive(Debug, Deserialize)]
pub struct LuaKeymapOpts {
    #[serde(default)]
    pub description: String,
}

#[derive(Debug)]
pub struct LuaKeymap {
    pub mode: Mode,
    pub keys: String,
    pub command: MappableCommand,
    pub options: LuaKeymapOpts,
}

pub fn setup_keymap_api(lua: &Lua, core: &Table, runtime_sender: UnboundedSender<RuntimeMessage>) -> Result<()> {
    core.set(
        "set_keymap_command",
        lua.create_function(move |lua: &Lua, args: (String, String, String, Table)| {
            set_keymap_command(lua, args, runtime_sender.clone())
        })?,
    )?;

    Ok(())
}

pub fn set_keymap_command(
    lua: &Lua,
    (mode, keys, command, opts): (String, String, String, Table),
    runtime_sender: UnboundedSender<RuntimeMessage>,
) -> mlua::Result<()> {
    let options = lua.from_value::<LuaKeymapOpts>(Value::Table(opts))?;

    let keymap = LuaKeymap {
        mode: mode.into(),
        keys,
        command: command.into(),
        options,
    };

    runtime_sender.send(RuntimeMessage::SetKeymap(keymap)).ok();

    Ok(())
}
