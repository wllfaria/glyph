use glyph_config::GlyphConfig;
use glyph_core::editor::Editor;
use glyph_core::rect::Point;
use glyph_core::syntax::Highlighter;

use crate::backend::CursorKind;
use crate::buffer::Buffer;

pub trait RenderLayer {
    fn draw(&self, buffer: &mut Buffer, ctx: &mut DrawContext, config: GlyphConfig);

    #[allow(unused_variables)]
    fn cursor(&self, editor: &Editor, config: GlyphConfig) -> (Option<Point>, CursorKind) {
        (None, CursorKind::Hidden)
    }
}

#[derive(Debug)]
pub struct DrawContext<'ctx> {
    pub editor: &'ctx Editor,
    pub highlighter: &'ctx Highlighter,
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

    pub fn cursor(&self, editor: &Editor, config: GlyphConfig) -> (Option<Point>, CursorKind) {
        for layer in &self.layers {
            if let (Some(pos), kind) = layer.cursor(editor, config) {
                return (Some(pos), kind);
            }
        }
        (None, CursorKind::Hidden)
    }

    pub fn draw_frame(&mut self, buffer: &mut Buffer, ctx: &mut DrawContext, config: GlyphConfig) {
        for layer in &mut self.layers {
            layer.draw(buffer, ctx, config);
        }
    }
}
