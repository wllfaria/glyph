use crate::config::{Config, LineNumbers};
use crate::pane::gutter::Gutter;
use crate::pane::Viewport;
use crate::theme::Theme;

#[derive(Debug)]
pub struct RelativeLineDrawer {
    config: Config,
    theme: Theme,
}

impl RelativeLineDrawer {
    pub fn new(config: Config, theme: Theme) -> Self {
        Self { config, theme }
    }
}

impl Gutter for RelativeLineDrawer {
    fn draw(&self, viewport: &mut Viewport, total_lines: usize, line: usize, scroll: usize) {
        let total_lines = usize::min(viewport.height, total_lines);
        let normalized_line = line + 1;
        let mut scroll_row = scroll;
        let style = &self.theme.gutter;

        for y in 0..total_lines {
            scroll_row += 1;
            let mut line = usize::abs_diff(scroll_row, normalized_line).to_string();

            if let LineNumbers::RelativeNumbered = self.config.line_numbers {
                println!("idk why");
                match normalized_line {
                    l if l == scroll_row => line = scroll_row.to_string(),
                    _ => (),
                }
            }

            line = " ".repeat(self.config.gutter_width - 1 - line.len()) + &line;
            line.push(' ');

            for (x, c) in line.chars().enumerate() {
                viewport.set_cell(x, y, c, style);
            }
        }

        if total_lines < viewport.height {
            let mut line = " ".repeat(self.config.gutter_width - 2);
            line.push(self.config.empty_line_char);
            line.push(' ');

            for y in total_lines..viewport.height {
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
    use crate::config::{Config, LineNumbers};
    use crate::theme::Theme;
    use crate::viewport::Viewport;

    #[test]
    fn test_draw_gutter() {
        let mut vp = Viewport::new(6, 5);
        let theme = Theme::default();
        let config = Config::default();
        let relative_gutter = RelativeLineDrawer::new(config, theme);

        relative_gutter.draw(&mut vp, 3, 2, 0);

        assert_eq!(vp.cells[4].c, '2');
        assert_eq!(vp.cells[10].c, '1');
        assert_eq!(vp.cells[16].c, '0');
        assert_eq!(vp.cells[22].c, '~');
        assert_eq!(vp.cells[28].c, '~');
    }

    #[test]
    fn test_draw_with_scroll() {
        let mut vp = Viewport::new(6, 100);
        let theme = Theme::default();
        let config = Config::default();
        let relative_gutter = RelativeLineDrawer::new(config, theme);

        relative_gutter.draw(&mut vp, 400, 103, 103);

        // 103 scrolled lines, should be 0 when rendered
        assert_eq!(vp.cells[0].c, ' ');
        assert_eq!(vp.cells[1].c, ' ');
        assert_eq!(vp.cells[2].c, ' ');
        assert_eq!(vp.cells[3].c, ' ');
        assert_eq!(vp.cells[4].c, '0');
        assert_eq!(vp.cells[5].c, ' ');
    }

    #[test]
    fn test_draw_with_scroll_numbered() {
        let mut vp = Viewport::new(6, 100);
        let theme = Theme::default();
        let mut config = Config::default();
        config.line_numbers = LineNumbers::RelativeNumbered;
        let relative_gutter = RelativeLineDrawer::new(config, theme);

        relative_gutter.draw(&mut vp, 400, 103, 103);

        // 103 scrolled lines, should be 104 when rendered
        assert_eq!(vp.cells[0].c, ' ');
        assert_eq!(vp.cells[1].c, ' ');
        assert_eq!(vp.cells[2].c, '1');
        assert_eq!(vp.cells[3].c, '0');
        assert_eq!(vp.cells[4].c, '4');
        assert_eq!(vp.cells[5].c, ' ');
    }
}
