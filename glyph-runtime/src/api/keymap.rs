use glyph_core::command::MappableCommand;
use mlua::{Function, Lua, LuaSerdeExt, Table, Value};
use tokio::sync::mpsc::UnboundedSender;

use crate::config::keymap::{LuaKeymapConfig, LuaKeymapOptions, LuaMappableCommand};
use crate::error::Result;
use crate::RuntimeMessage;

pub fn setup_keymap_api(
    lua: &Lua,
    core: &Table,
    runtime_sender: UnboundedSender<RuntimeMessage<'static>>,
) -> Result<()> {
    let sender = runtime_sender.clone();
    core.set(
        "keymap_command_set",
        lua.create_function(move |lua: &Lua, args: (String, String, String, Table)| {
            let sender = sender.clone();
            keymap_command_set(lua, args, sender)
        })?,
    )?;

    let sender = runtime_sender.clone();
    core.set(
        "keymap_function_set",
        lua.create_function(move |lua: &Lua, args: (String, String, Function, Table)| {
            let sender = sender.clone();
            keymap_function_set(lua, args, sender)
        })?,
    )?;

    Ok(())
}

pub fn keymap_command_set(
    lua: &Lua,
    (mode, keys, command, options): (String, String, String, Table),
    runtime_sender: UnboundedSender<RuntimeMessage<'_>>,
) -> mlua::Result<()> {
    let options = lua.from_value::<LuaKeymapOptions>(Value::Table(options))?;

    let command = MappableCommand::STATIC_CMD_LIST
        .iter()
        .find(|cmd| match cmd {
            MappableCommand::Static { name, .. } => name == &command,
            MappableCommand::Dynamic { .. } => false,
        })
        .unwrap();

    let keymap = LuaKeymapConfig {
        mode: mode.into(),
        keys,
        command: LuaMappableCommand::Borrowed(command),
        options,
    };

    runtime_sender.send(RuntimeMessage::SetKeymap(keymap)).ok();

    Ok(())
}

pub fn keymap_function_set(
    lua: &Lua,
    (mode, keys, command, options): (String, String, Function, Table),
    runtime_sender: UnboundedSender<RuntimeMessage<'static>>,
) -> mlua::Result<()> {
    let options = lua.from_value::<LuaKeymapOptions>(Value::Table(options))?;

    let sender = runtime_sender.clone();
    let keymap = LuaKeymapConfig {
        mode: mode.into(),
        keys,
        command: LuaMappableCommand::Owned(MappableCommand::Dynamic {
            callback: Box::new(move || match command.call::<()>(()) {
                Ok(_) => {}
                Err(err) => _ = sender.send(RuntimeMessage::Error(err.to_string())).ok(),
            }),
        }),
        options,
    };

    runtime_sender.send(RuntimeMessage::SetKeymap(keymap)).ok();

    Ok(())
}
