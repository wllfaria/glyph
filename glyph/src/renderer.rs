use glyph_term::buffer::Buffer;

use crate::editor::Editor;

pub trait RenderLayer {
    fn draw(&self, buffer: &mut Buffer, ctx: &mut DrawContext);
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

    pub fn draw_frame(&mut self, buffer: &mut Buffer, ctx: &mut DrawContext) {
        for layer in &mut self.layers {
            layer.draw(buffer, ctx);
        }
    }
}
