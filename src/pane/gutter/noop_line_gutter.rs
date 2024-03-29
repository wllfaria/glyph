use crate::config::Config;
use crate::pane::Gutter;
use crate::pane::Viewport;
use crate::theme::Theme;

#[derive(Debug)]
pub struct NoopLineDrawer {
    _config: Config,
    _theme: Theme,
}

impl NoopLineDrawer {
    pub fn new(config: Config, theme: Theme) -> Self {
        Self {
            _config: config,
            _theme: theme,
        }
    }
}

impl Gutter for NoopLineDrawer {
    fn draw(&self, _: &mut Viewport, _: usize, _: usize, _: usize) {}
}
