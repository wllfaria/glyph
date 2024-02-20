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
    fn draw(&self, _: &mut Viewport, _: usize, _: usize, _: usize) {}
}
