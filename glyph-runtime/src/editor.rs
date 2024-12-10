use std::sync::Arc;

use mlua::{Lua, Table};
use parking_lot::RwLock;
use tokio::sync::mpsc::UnboundedSender;

use crate::error::Result;
use crate::{GlyphContext, RuntimeMessage};

pub fn setup_editor_api(
    lua: &Lua,
    core: &Table,
    _runtime_sender: UnboundedSender<RuntimeMessage<'static>>,
    context: Arc<RwLock<GlyphContext>>,
) -> Result<()> {
    core.set(
        "get_editor_mode",
        lua.create_function(move |_: &Lua, _: ()| get_editor_mode(context.clone()))?,
    )?;

    Ok(())
}

fn get_editor_mode(context: Arc<RwLock<GlyphContext>>) -> mlua::Result<String> {
    Ok(context.read().editor.read().mode().to_string())
}
