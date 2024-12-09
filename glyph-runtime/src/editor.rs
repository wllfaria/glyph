use mlua::{Lua, Table};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::error::Result;
use crate::{RuntimeMessage, RuntimeQuery};

pub fn setup_editor_api(
    lua: &Lua,
    core: &Table,
    runtime_sender: UnboundedSender<RuntimeMessage<'static>>,
) -> Result<()> {
    core.set(
        "get_editor_mode",
        lua.create_function(move |_: &Lua, _: ()| get_editor_mode(runtime_sender.clone()))?,
    )?;

    Ok(())
}

fn get_editor_mode(runtime_sender: UnboundedSender<RuntimeMessage<'_>>) -> mlua::Result<String> {
    todo!();
}
