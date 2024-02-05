use std::io;

use crate::pane::{line_drawer, PaneDimensions};

#[derive(Debug)]
pub struct NoopLineDrawer {}

impl NoopLineDrawer {
    pub fn new() -> Self {
        Self {}
    }
}

impl line_drawer::LineDrawer for NoopLineDrawer {
    fn draw_lines(&mut self, _: &PaneDimensions, _: u16, _: u16, _: u16) -> io::Result<()> {
        Ok(())
    }
}
