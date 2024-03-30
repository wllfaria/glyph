use crate::config::Config;
use crate::pane::Frame;
use crate::theme::Theme;

use crate::pane::gutter::Gutter;
use crate::tui::rect::Rect;

#[derive(Debug, Clone)]
pub struct AbsoluteLineGutter<'a> {
    config: &'a Config,
    theme: &'a Theme,
    area: Rect,
}

impl<'a> AbsoluteLineGutter<'a> {
    pub fn new(config: &'a Config, theme: &'a Theme, area: Rect) -> Self {
        Self {
            config,
            theme,
            area,
        }
    }
}

impl Gutter for AbsoluteLineGutter<'_> {
    fn draw(&self, viewport: &mut Frame, total_lines: usize, _: usize, scroll: usize) {
        let total_lines = usize::min(self.area.height.into(), total_lines);
        let mut scroll = scroll;
        let style = &self.theme.gutter;

        for y in 0..total_lines {
            scroll += 1;
            let mut line = scroll.to_string();
            line = " ".repeat(self.config.gutter_width - 1 - line.len()) + &line;
            line.push(' ');

            for (x, c) in line.chars().enumerate() {
                viewport.set_cell(x as u16, y as u16, c, style);
            }
        }

        if total_lines < self.area.height.into() {
            for y in total_lines..viewport.height.into() {
                let mut line = " ".repeat(self.config.gutter_width - 2);
                line.push(self.config.empty_line_char);
                line.push(' ');

                for (x, c) in line.chars().enumerate() {
                    viewport.set_cell(x as u16, y as u16, c, style);
                }
            }
        }
    }

    fn width(&self) -> u16 {
        self.config.gutter_width as u16
    }
}
