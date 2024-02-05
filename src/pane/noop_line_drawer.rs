use std::io;

use super::{line_drawer::LineDrawer, PaneDimensions};

pub struct NoopLineDrawer {}

impl NoopLineDrawer {
    pub fn new() -> Self {
        Self {}
    }
}

impl LineDrawer for NoopLineDrawer {
    fn draw_lines(&mut self, _: &PaneDimensions, _: u16, _: u16) -> io::Result<()> {
        Ok(())
    }
}
