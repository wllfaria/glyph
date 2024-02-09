use std::io::{self, stdout};

use crossterm::cursor;
use crossterm::style::{Color, Print, Stylize};
use crossterm::QueueableCommand;

use crate::config::{Config, LineNumbers};
use crate::pane::line_drawer::LineDrawer;
use crate::pane::PaneDimensions;

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

impl LineDrawer for RelativeLineDrawer {
    fn draw_lines(
        &mut self,
        dimensions: &PaneDimensions,
        total_lines: u16,
        current_line: u16,
        scroll_row: u16,
    ) -> io::Result<()> {
        let total_lines = u16::min(dimensions.height, total_lines);
        let normalized_line = current_line + 1;
        let mut scroll_row = scroll_row;

        for i in 0..total_lines {
            scroll_row += 1;
            let mut line = u16::abs_diff(scroll_row, normalized_line).to_string();

            match self.config.line_numbers {
                LineNumbers::RelativeNumbered => match normalized_line {
                    l if l == scroll_row => line = scroll_row.to_string(),
                    _ => (),
                },
                _ => (),
            }

            let offset = dimensions.col + self.config.sidebar_width - line.len() as u16;

            self.stdout
                .queue(cursor::MoveTo(offset, i as u16))?
                .queue(Print(line.with(Color::DarkGrey)))?;
        }

        if total_lines < dimensions.height {
            for i in total_lines..dimensions.height {
                let offset = dimensions.col + self.config.sidebar_width - 1;
                self.stdout
                    .queue(cursor::MoveTo(offset, i as u16))?
                    .queue(Print(self.config.empty_line_char))?;
            }
        }

        Ok(())
    }
}
