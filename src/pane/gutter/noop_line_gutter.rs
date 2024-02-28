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
    use crate::config::{Config, EditorBackground, Keys, LineNumbers};
    use crate::theme::Theme;
    use crate::viewport::Viewport;

    fn get_config() -> Config {
        Config {
            gutter_width: 6,
            theme: "".into(),
            keys: Keys::default(),
            log_file: None,
            background: EditorBackground::Dark,
            line_numbers: LineNumbers::Absolute,
            empty_line_char: '~',
            show_diagnostics: true,
            mouse_scroll_lines: None,
        }
    }

    #[test]
    fn test_ensure_does_nothing() {
        let mut vp = Viewport::new(2, 2);
        let theme = Theme::default();
        let config = get_config();
        let noop_gutter = NoopLineDrawer::new(config, theme);

        noop_gutter.draw(&mut vp, 2, 1, 1);

        assert_eq!(vp.cells[0].c, ' ');
        assert_eq!(vp.cells[1].c, ' ');
        assert_eq!(vp.cells[2].c, ' ');
        assert_eq!(vp.cells[3].c, ' ');
    }
}
