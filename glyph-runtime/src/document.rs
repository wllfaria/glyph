use std::sync::Arc;

use glyph_core::document::DocumentId;
use mlua::{ExternalError, Integer, Lua, Table};
use parking_lot::RwLock;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;

use crate::error::Result;
use crate::{GlyphContext, RuntimeMessage};

#[derive(Debug, Error)]
enum DocumentError {
    #[error("Document number cannot be negative")]
    NegativeDocument,
    #[error("Invalid Document ID: {0}")]
    InvalidDocument(usize),
}

pub fn setup_document_api(
    lua: &Lua,
    core: &Table,
    _runtime_sender: UnboundedSender<RuntimeMessage<'static>>,
    context: Arc<RwLock<GlyphContext>>,
) -> Result<()> {
    let inner_context = context.clone();
    core.set(
        "document_get_active",
        lua.create_function(move |_: &Lua, _: ()| document_get_active(inner_context.clone()))?,
    )?;

    let inner_context = context.clone();
    core.set(
        "document_is_valid",
        lua.create_function(move |_: &Lua, document: Integer| document_is_valid(document, inner_context.clone()))?,
    )?;

    let inner_context = context.clone();
    core.set(
        "document_get_line_count",
        lua.create_function(move |_: &Lua, document: Integer| {
            document_get_line_count(document, inner_context.clone())
        })?,
    )?;

    Ok(())
}
fn document_get_active(context: Arc<RwLock<GlyphContext>>) -> mlua::Result<usize> {
    let ctx = context.read();
    let editor = ctx.editor.read();
    let tab = editor.focused_tab();
    let window = tab.tree.focus();
    let window = tab.tree.window(window);
    Ok(window.document.into())
}

fn document_is_valid(document: Integer, context: Arc<RwLock<GlyphContext>>) -> mlua::Result<bool> {
    if document.is_negative() {
        return Ok(false);
    }

    let document = document as usize;
    if document == 0 {
        return Ok(true);
    }

    Ok(DocumentId::new(document)
        .map(|document| context.read().editor.read().get_document(&document).is_some())
        .unwrap_or_default())
}

fn document_get_line_count(document: Integer, context: Arc<RwLock<GlyphContext>>) -> mlua::Result<Integer> {
    if document.is_negative() {
        return Err(DocumentError::NegativeDocument.into_lua_err());
    }

    let document = document as usize;
    let context = context.read();
    let editor = context.editor.read();

    let lines = if document == 0 {
        let window = editor.focused_tab().tree.focus();
        let window = editor.focused_tab().tree.window(window);
        let document = window.document;
        let document = editor.document(&document);
        document.text().len_lines()
    } else {
        let Some(document) = DocumentId::new(document).and_then(|document| editor.get_document(&document)) else {
            return Err(DocumentError::InvalidDocument(document).into_lua_err());
        };
        document.text().len_lines()
    };

    Ok(lines as i64)
}
