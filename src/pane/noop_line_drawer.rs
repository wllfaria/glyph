use std::io;

use crate::pane::{LineDrawer, PaneDimensions};

#[derive(Debug)]
pub struct NoopLineDrawer {}

impl NoopLineDrawer {
    pub fn new() -> Self {
        Self {}
    }
}

impl LineDrawer for NoopLineDrawer {
    fn draw_lines(&mut self, _: &PaneDimensions, _: u16, _: u16, _: u16) -> io::Result<()> {
        Ok(())
    }
}
