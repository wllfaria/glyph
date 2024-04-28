use config::{Config, LineNumbers};
use theme::Theme;

use crate::{
    frame::Frame,
    tui::{gutter::Gutter, rect::Rect},
};

#[derive(Debug)]
pub struct RelativeLineDrawer<'a> {
    config: &'a Config,
    theme: &'a Theme,
    area: Rect,
}

impl<'a> RelativeLineDrawer<'a> {
    pub fn new(config: &'a Config, theme: &'a Theme, area: Rect) -> Self {
        Self {
            config,
            theme,
            area,
        }
    }
}

impl Gutter for RelativeLineDrawer<'_> {
    fn draw(&self, viewport: &mut Frame, total_lines: usize, line: usize, scroll: usize) {
        let total_lines = usize::min(self.area.height.into(), total_lines);
        let normalized_line = line + 1;
        let mut scroll = scroll;
        let style = &self.theme.gutter;

        for y in 0..total_lines {
            scroll += 1;
            let mut line = usize::abs_diff(scroll, normalized_line).to_string();

            if let LineNumbers::RelativeNumbered = self.config.line_numbers {
                match normalized_line {
                    l if l == scroll => line = scroll.to_string(),
                    _ => (),
                }
            }

            line = if normalized_line == scroll {
                String::from(" ") + &line + &" ".repeat(self.config.gutter_width - 2 - line.len())
            } else {
                " ".repeat(self.config.gutter_width - 1 - line.len()) + &line
            };
            line = " ".repeat(self.config.gutter_width - 1 - line.len()) + &line + " ";

            viewport.set_text(self.area.x, self.area.y + y as u16, &line, style);
        }

        if total_lines < self.area.height.into() {
            let mut line = " ".repeat(self.config.gutter_width - 2);
            line.push(self.config.empty_line_char);
            line.push(' ');

            for y in total_lines..self.area.height.into() {
                viewport.set_text(self.area.x, self.area.y + y as u16, &line, style);
            }
        }
    }

    fn width(&self) -> u16 {
        self.config.gutter_width as u16
    }
}
