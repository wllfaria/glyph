use crate::config::{Config, LineNumbers};
use crate::pane::gutter::Gutter;
use crate::pane::Frame;
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
    fn draw(&self, viewport: &mut Frame, total_lines: usize, line: usize, scroll: usize) {
        let total_lines = usize::min(viewport.height.into(), total_lines);
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
                viewport.set_cell(x as u16, y as u16, c, style);
            }
        }

        if total_lines < viewport.height.into() {
            let mut line = " ".repeat(self.config.gutter_width - 2);
            line.push(self.config.empty_line_char);
            line.push(' ');

            for y in total_lines..viewport.height.into() {
                for (x, c) in line.chars().enumerate() {
                    viewport.set_cell(x as u16, y as u16, c, style);
                }
            }
        }
    }
}
