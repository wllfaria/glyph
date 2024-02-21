use crate::config::Config;
use crate::pane::Gutter;
use crate::pane::Viewport;
use crate::theme::Theme;

#[derive(Debug)]
pub struct NoopLineDrawer {
    config: Config,
    theme: Theme,
}

impl<'a> NoopLineDrawer {
    pub fn new(config: Config, theme: Theme) -> Self {
        Self { config, theme }
    }
}

impl Gutter for NoopLineDrawer {
    fn draw(&self, _: &mut Viewport, _: usize, _: usize, _: usize) {}
}
