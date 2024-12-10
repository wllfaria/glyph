use std::sync::Arc;

use glyph_core::window::WindowId;
use mlua::{ExternalError, Integer, Lua, Table};
use parking_lot::RwLock;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;

use crate::error::Result;
use crate::{GlyphContext, RuntimeMessage};

#[derive(Debug, Error)]
enum WindowError {
    #[error("Window number cannot be negative")]
    NegativeWindow,
    #[error("Invalid window ID: {0}")]
    InvalidWindow(usize),
}

pub fn setup_window_api(
    lua: &Lua,
    core: &Table,
    _runtime_sender: UnboundedSender<RuntimeMessage<'static>>,
    context: Arc<RwLock<GlyphContext>>,
) -> Result<()> {
    let inner_context = context.clone();
    core.set(
        "window_get_active",
        lua.create_function(move |_: &Lua, _: ()| window_get_active(inner_context.clone()))?,
    )?;

    let inner_context = context.clone();
    core.set(
        "window_is_valid",
        lua.create_function(move |_: &Lua, window: Integer| window_is_valid(window, inner_context.clone()))?,
    )?;

    let inner_context = context.clone();
    core.set(
        "window_get_cursor",
        lua.create_function(move |lua: &Lua, window: Integer| window_get_cursor(lua, window, inner_context.clone()))?,
    )?;

    Ok(())
}

fn window_get_active(context: Arc<RwLock<GlyphContext>>) -> mlua::Result<usize> {
    Ok(context.read().editor.read().focused_tab().tree.focus().into())
}

fn window_is_valid(window: Integer, context: Arc<RwLock<GlyphContext>>) -> mlua::Result<bool> {
    if window.is_negative() {
        return Ok(false);
    }

    let window = window as usize;
    if window == 0 {
        return Ok(true);
    }

    Ok(WindowId::new(window)
        .map(|window| {
            context
                .read()
                .editor
                .read()
                .focused_tab()
                .tree
                .nodes()
                .contains_key(&window)
        })
        .unwrap_or_default())
}

fn create_cursor_table(lua: &Lua, x: usize, y: usize) -> mlua::Result<Table> {
    let cursor = lua.create_table()?;
    cursor.set("x", x)?;
    cursor.set("y", y)?;
    Ok(cursor)
}

fn window_get_cursor(lua: &Lua, window: Integer, context: Arc<RwLock<GlyphContext>>) -> mlua::Result<Table> {
    if window.is_negative() {
        return Err(WindowError::NegativeWindow.into_lua_err());
    }

    let window = window as usize;
    let context = context.read();
    let cursors = context.cursors.read();

    let cursor = if window == 0 {
        let editor = context.editor.read();
        let tab = editor.focused_tab();
        let window = tab.tree.focus();

        cursors
            .get(&window)
            .expect("we don't have an active window, the editor must be closed")
    } else {
        let Some(cursor) = WindowId::new(window).and_then(|window| cursors.get(&window)) else {
            return Err(WindowError::InvalidWindow(window).into_lua_err());
        };
        cursor
    };

    let cursor = create_cursor_table(lua, cursor.x(), cursor.y())?;

    Ok(cursor)
}
