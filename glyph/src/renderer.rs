use glyph_term::backend::CursorKind;
use glyph_term::buffer::Buffer;

use crate::cursor::Cursor;
use crate::editor::Editor;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Anchor {
    pub x: u16,
    pub y: u16,
}

impl From<Cursor> for Anchor {
    fn from(value: Cursor) -> Anchor {
        Anchor {
            x: value.x() as u16,
            y: value.y() as u16,
        }
    }
}

pub trait RenderLayer {
    fn draw(&self, buffer: &mut Buffer, ctx: &mut DrawContext);

    #[allow(unused_variables)]
    fn cursor(&self, editor: &Editor) -> (Option<Anchor>, CursorKind) {
        (None, CursorKind::Hidden)
    }
}

#[derive(Debug)]
pub struct DrawContext<'ctx> {
    pub editor: &'ctx Editor,
}

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

    pub fn cursor(&self, editor: &Editor) -> (Option<Anchor>, CursorKind) {
        for layer in &self.layers {
            if let (Some(pos), kind) = layer.cursor(editor) {
                return (Some(pos), kind);
            }
        }
        (None, CursorKind::Hidden)
    }

    pub fn draw_frame(&mut self, buffer: &mut Buffer, ctx: &mut DrawContext) {
        for layer in &mut self.layers {
            layer.draw(buffer, ctx);
        }
    }
}
