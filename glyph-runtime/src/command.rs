use std::sync::Arc;

use mlua::{Lua, Table};
use parking_lot::RwLock;
use tokio::sync::mpsc::UnboundedSender;

use crate::error::Result;
use crate::{GlyphContext, RuntimeMessage};

pub fn setup_command_api(
    lua: &Lua,
    core: &Table,
    runtime_sender: UnboundedSender<RuntimeMessage<'static>>,
    _context: Arc<RwLock<GlyphContext>>,
) -> Result<()> {
    core.set(
        "user_command_create",
        lua.create_function(move |_: &Lua, args: (mlua::String, mlua::Function)| {
            user_command_create(args, runtime_sender.clone())
        })?,
    )?;

    Ok(())
}

fn user_command_create(
    (name, callback): (mlua::String, mlua::Function),
    runtime_sender: UnboundedSender<RuntimeMessage<'static>>,
) -> mlua::Result<()> {
    let name = name.to_string_lossy();

    runtime_sender
        .send(RuntimeMessage::UserCommandCreate(name, callback))
        .ok();

    Ok(())
}
