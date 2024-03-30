use crate::config::Config;
use crate::pane::Frame;
use crate::pane::Gutter;
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
    fn draw(&self, _: &mut Frame, _: usize, _: usize, _: usize) {}
}
