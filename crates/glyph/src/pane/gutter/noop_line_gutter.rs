use std::io;

use crate::pane::Gutter;
use crate::pane::PaneDimensions;

#[derive(Debug)]
pub struct NoopLineDrawer {}

impl NoopLineDrawer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Gutter for NoopLineDrawer {
    fn draw(&mut self, _: &PaneDimensions, _: u16, _: u16, _: u16) -> io::Result<()> {
        Ok(())
    }
}
