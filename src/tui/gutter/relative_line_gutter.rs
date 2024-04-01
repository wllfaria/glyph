use crate::config::{Config, LineNumbers};
use crate::frame::Frame;
use crate::theme::Theme;
use crate::tui::rect::Rect;

use super::Gutter;

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
        let mut scroll_row = scroll;
        let style = &self.theme.gutter;

        for y in 0..total_lines {
            scroll_row += 1;
            let mut line = usize::abs_diff(scroll_row, normalized_line).to_string();

            if let LineNumbers::RelativeNumbered = self.config.line_numbers {
                match normalized_line {
                    l if l == scroll_row => line = scroll_row.to_string(),
                    _ => (),
                }
            }

            line = " ".repeat(self.config.gutter_width - 1 - line.len()) + &line;
            line.push(' ');

            for (x, c) in line.chars().enumerate() {
                viewport.set_cell(self.area.x + x as u16, y as u16, c, style);
            }
        }

        if total_lines < self.area.height.into() {
            let mut line = " ".repeat(self.config.gutter_width - 2);
            line.push(self.config.empty_line_char);
            line.push(' ');

            for y in total_lines..viewport.height.into() {
                for (x, c) in line.chars().enumerate() {
                    viewport.set_cell(self.area.x + x as u16, y as u16, c, style);
                }
            }
        }
    }

    fn width(&self) -> u16 {
        self.config.gutter_width as u16
    }
}
