use std::sync::Arc;

use mlua::{Lua, LuaSerdeExt, Table};
use parking_lot::RwLock;
use serde::Deserialize;
use tokio::sync::mpsc::UnboundedSender;

use crate::error::Result;
use crate::{GlyphContext, RuntimeMessage};

pub fn setup_editor_api(
    lua: &Lua,
    core: &Table,
    runtime_sender: UnboundedSender<RuntimeMessage<'static>>,
    context: Arc<RwLock<GlyphContext>>,
) -> Result<()> {
    core.set(
        "editor_get_mode",
        lua.create_function(move |_: &Lua, _: ()| editor_get_mode(context.clone()))?,
    )?;

    let sender = runtime_sender.clone();
    core.set(
        "editor_quit",
        lua.create_function(move |lua: &Lua, args: Table| editor_quit(lua, args, sender.clone()))?,
    )?;

    let sender = runtime_sender.clone();
    core.set(
        "editor_write",
        lua.create_function(move |lua: &Lua, args: Table| editor_write(lua, args, sender.clone()))?,
    )?;

    Ok(())
}

fn editor_get_mode(context: Arc<RwLock<GlyphContext>>) -> mlua::Result<String> {
    Ok(context.read().editor.read().mode().to_string())
}

#[derive(Debug, Deserialize)]
pub struct QuitOptions {
    #[serde(default)]
    pub force: bool,
    #[serde(default)]
    pub all: bool,
}

fn editor_quit(
    lua: &Lua,
    options: Table,
    runtime_sender: UnboundedSender<RuntimeMessage<'static>>,
) -> mlua::Result<()> {
    let options = lua.from_value::<QuitOptions>(mlua::Value::Table(options))?;
    runtime_sender.send(RuntimeMessage::Quit(options)).ok();
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct WriteOptions {
    #[serde(default)]
    pub force: bool,
    #[serde(default)]
    pub all: bool,
}

fn editor_write(
    lua: &Lua,
    options: Table,
    runtime_sender: UnboundedSender<RuntimeMessage<'static>>,
) -> mlua::Result<()> {
    let options = lua.from_value::<WriteOptions>(mlua::Value::Table(options))?;
    runtime_sender.send(RuntimeMessage::Write(options)).ok();

    Ok(())
}
