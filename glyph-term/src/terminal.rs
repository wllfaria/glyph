use glyph_core::config::GlyphConfig;
use glyph_core::rect::Rect;

use crate::backend::Backend;
use crate::buffer::Buffer;

#[derive(Debug)]
pub struct Terminal<B: Backend> {
    pub backend: B,
    buffers: [Buffer; 2],
}

impl<B: Backend> Terminal<B> {
    pub fn new(backend: B) -> Terminal<B> {
        let area = backend.area().expect("couldn't get terminal size");
        let buffers = [Buffer::new(area), Buffer::new(area)];
        Terminal { backend, buffers }
    }

    pub fn current_buffer(&mut self) -> &mut Buffer {
        &mut self.buffers[0]
    }

    pub fn swap_buffers(&mut self) {
        self.buffers.swap(0, 1)
    }

    pub fn flush(&mut self, config: GlyphConfig<'_>) -> Result<(), std::io::Error> {
        let buffer = &self.buffers[0];
        let diffs = buffer.diff(&self.buffers[1]);
        self.backend.draw(diffs.into_iter(), config)?;
        self.backend.flush()
    }

    pub fn resize(&mut self, new_area: Rect) {
        self.buffers.iter_mut().for_each(|b| b.resize(new_area));
    }
}
