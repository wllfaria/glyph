use std::collections::BTreeMap;
use std::sync::Arc;

use crossterm::event::Event;
use glyph_core::config::GlyphConfig;
use glyph_core::cursor::Cursor;
use glyph_core::editor::{Editor, EventResult};
use glyph_core::rect::Point;
use glyph_core::syntax::Highlighter;
use glyph_core::window::WindowId;
use mlua::Lua;
use parking_lot::RwLock;

use crate::backend::CursorKind;
use crate::buffer::Buffer;

pub trait RenderLayer {
    fn draw(&self, buffer: &mut Buffer, ctx: &mut Context<'_>, config: GlyphConfig<'_>);

    #[allow(unused_variables)]
    fn handle_event(
        &self,
        event: &Event,
        ctx: &mut Context<'_>,
        config: GlyphConfig<'_>,
    ) -> Result<Option<EventResult>, std::io::Error> {
        Ok(None)
    }

    #[allow(unused_variables)]
    fn cursor(&self, editor: &mut Context<'_>, config: GlyphConfig<'_>) -> (Option<Point>, CursorKind) {
        (None, CursorKind::Hidden)
    }
}

#[derive(Debug)]
pub struct Context<'ctx> {
    pub editor: Arc<RwLock<Editor>>,
    pub cursors: Arc<RwLock<BTreeMap<WindowId, Cursor>>>,
    pub runtime: &'ctx Lua,
    pub highlighter: &'ctx mut Highlighter,
    pub config: GlyphConfig<'ctx>,
}

#[derive(Default)]
pub struct Renderer {
    layers: Vec<Box<dyn RenderLayer>>,
}

impl std::fmt::Debug for Renderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Renderer").finish()
    }
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer { layers: Vec::default() }
    }

    pub fn push_layer(&mut self, layer: Box<dyn RenderLayer>) {
        self.layers.push(layer)
    }

    pub fn cursor(&self, ctx: &mut Context<'_>, config: GlyphConfig<'_>) -> (Option<Point>, CursorKind) {
        for layer in &self.layers {
            if let (Some(pos), kind) = layer.cursor(ctx, config) {
                return (Some(pos), kind);
            }
        }
        (None, CursorKind::Hidden)
    }

    pub fn handle_event(
        &self,
        event: &Event,
        ctx: &mut Context<'_>,
        config: GlyphConfig<'_>,
    ) -> Result<Option<EventResult>, std::io::Error> {
        for layer in &self.layers {
            match layer.handle_event(event, ctx, config)? {
                Some(EventResult::Consumed(_)) => {}
                Some(EventResult::Ignored(_)) => {}
                None => {}
            }
        }

        Ok(None)
    }

    pub fn draw_frame(&mut self, buffer: &mut Buffer, ctx: &mut Context<'_>, config: GlyphConfig<'_>) {
        for layer in &mut self.layers {
            layer.draw(buffer, ctx, config);
        }
    }
}
