use crate::config::Config;
use crate::pane::Viewport;
use crate::theme::Theme;

use crate::pane::gutter::Gutter;

#[derive(Debug, Clone)]
pub struct AbsoluteLineGutter {
    config: Config,
    theme: Theme,
}

impl AbsoluteLineGutter {
    pub fn new(config: Config, theme: Theme) -> Self {
        Self { config, theme }
    }
}

impl Gutter for AbsoluteLineGutter {
    fn draw(&self, viewport: &mut Viewport, total_lines: usize, _: usize, scroll: usize) {
        let total_lines = usize::min(viewport.height, total_lines);
        let mut scroll = scroll;
        let style = &self.theme.gutter;

        for y in 0..total_lines {
            scroll += 1;
            let mut line = scroll.to_string();
            line = " ".repeat(self.config.gutter_width - 1 - line.len()) + &line;
            line.push(' ');

            for (x, c) in line.chars().enumerate() {
                viewport.set_cell(x, y, c, style);
            }
        }

        if total_lines < viewport.height {
            for y in total_lines..viewport.height {
                let mut line = " ".repeat(self.config.gutter_width - 2);
                line.push(self.config.empty_line_char);
                line.push(' ');

                for (x, c) in line.chars().enumerate() {
                    viewport.set_cell(x, y, c, style);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::theme::Theme;
    use crate::viewport::Viewport;

    #[test]
    fn test_draw_gutter() {
        let mut vp = Viewport::new(6, 5);
        let theme = Theme::default();
        let config = Config::default();
        let absolute_gutter = AbsoluteLineGutter::new(config, theme);

        absolute_gutter.draw(&mut vp, 3, 2, 0);

        assert_eq!(vp.cells[4].c, '1');
        assert_eq!(vp.cells[10].c, '2');
        assert_eq!(vp.cells[16].c, '3');
        assert_eq!(vp.cells[22].c, '~');
        assert_eq!(vp.cells[28].c, '~');
    }

    #[test]
    fn test_draw_with_scroll() {
        let mut vp = Viewport::new(6, 100);
        let theme = Theme::default();
        let config = Config::default();
        let absolute_gutter = AbsoluteLineGutter::new(config, theme);

        absolute_gutter.draw(&mut vp, 400, 0, 103);

        // 103 scrolled lines, should start at 104. and preserve the gap
        assert_eq!(vp.cells[0].c, ' ');
        assert_eq!(vp.cells[1].c, ' ');
        assert_eq!(vp.cells[2].c, '1');
        assert_eq!(vp.cells[3].c, '0');
        assert_eq!(vp.cells[4].c, '4');
        assert_eq!(vp.cells[5].c, ' ');
    }
}
