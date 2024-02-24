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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::theme::Theme;
    use crate::viewport::Viewport;

    #[test]
    fn test_ensure_does_nothing() {
        let mut vp = Viewport::new(2, 2);
        let theme = Theme::default();
        let config = Config::default();
        let noop_gutter = NoopLineDrawer::new(config, theme);

        noop_gutter.draw(&mut vp, 2, 1, 1);

        assert_eq!(vp.cells[0].c, ' ');
        assert_eq!(vp.cells[1].c, ' ');
        assert_eq!(vp.cells[2].c, ' ');
        assert_eq!(vp.cells[3].c, ' ');
    }
}
