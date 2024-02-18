use std::io::{self, stdout};

use crossterm::cursor;
use crossterm::style::{self, Color, Print, Stylize};
use crossterm::QueueableCommand;

use crate::config::{Config, LineNumbers};
use crate::pane::gutter::Gutter;
use crate::pane::PaneDimensions;
use crate::theme::Theme;

#[derive(Debug)]
pub struct RelativeLineDrawer {
    stdout: io::Stdout,
    config: &'static Config,
}

impl RelativeLineDrawer {
    pub fn new() -> Self {
        Self {
            stdout: stdout(),
            config: Config::get(),
        }
    }
}

impl Gutter for RelativeLineDrawer {
    fn draw(
        &mut self,
        dimensions: &PaneDimensions,
        total_lines: u16,
        current_line: u16,
        scroll_row: u16,
    ) -> io::Result<()> {
        let total_lines = u16::min(dimensions.height, total_lines);
        let normalized_line = current_line + 1;
        let mut scroll_row = scroll_row;
        let theme = Theme::get();

        for i in 0..total_lines {
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

            self.stdout
                .queue(cursor::MoveTo(dimensions.col, i))?
                .queue(style::SetBackgroundColor(theme.style.bg.unwrap()))?
                .queue(Print(line.with(Color::DarkGrey)))?;
        }

        if total_lines < dimensions.height {
            for i in total_lines..dimensions.height {
                self.stdout
                    .queue(cursor::MoveTo(dimensions.col, i))?
                    .queue(Print(self.config.empty_line_char))?;
            }
        }

        Ok(())
    }
}
