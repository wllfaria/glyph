use crate::config::{Config, LineNumbers};
use crate::pane::gutter::Gutter;
use crate::pane::Viewport;
use crate::theme::Theme;

#[derive(Debug)]
pub struct RelativeLineDrawer {
    config: &'static Config,
}

impl RelativeLineDrawer {
    pub fn new() -> Self {
        Self {
            config: Config::get(),
        }
    }
}

impl Gutter for RelativeLineDrawer {
    fn draw(&mut self, viewport: &mut Viewport, total_lines: usize, line: u16, scroll: u16) {
        let total_lines = usize::min(viewport.dimensions.height as usize, total_lines);
        let normalized_line = line + 1;
        let mut scroll_row = scroll;
        let theme = Theme::get();

        for y in 0..total_lines {
            scroll_row += 1;
            let mut line = u16::abs_diff(scroll_row, normalized_line).to_string();

            if let LineNumbers::RelativeNumbered = self.config.line_numbers {
                match normalized_line {
                    l if l == scroll_row => line = scroll_row.to_string(),
                    _ => (),
                }
            }

            line = " ".repeat(self.config.gutter_width as usize - 1 - line.len()) + &line;
            line.push_str(" ");

            for (x, c) in line.chars().enumerate() {
                viewport.set_cell(x, y, c, &Theme::get().style);
            }
        }

        if total_lines < viewport.dimensions.height as usize {
            let mut line = " ".repeat(self.config.gutter_width as usize - 2);
            line.push(self.config.empty_line_char);
            line.push(' ');

            for y in total_lines..viewport.dimensions.height as usize {
                for (x, c) in line.chars().enumerate() {
                    viewport.set_cell(x, y, c, &Theme::get().style);
                }
            }
        }
    }
}
