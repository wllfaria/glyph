use crate::pane::Gutter;
use crate::pane::Viewport;

#[derive(Debug)]
pub struct NoopLineDrawer {}

impl NoopLineDrawer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Gutter for NoopLineDrawer {
    fn draw(&mut self, _: &mut Viewport, _: usize, _: u16, _: u16) {}
}
