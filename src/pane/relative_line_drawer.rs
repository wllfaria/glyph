use std::io::{self, stdout};

use crossterm::cursor;
use crossterm::style::{Color, Print, Stylize};
use crossterm::QueueableCommand;

use crate::config::{Config, LineNumbers};
use crate::pane::line_drawer::LineDrawer;

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
        dimensions: &super::PaneDimensions,
        total_lines: u16,
        current_line: u16,
    ) -> io::Result<()> {
        let total_lines = u16::min(dimensions.height, total_lines);
        let normalized_line = current_line + 1;

        for i in 0..total_lines {
            let normalized_row = i + 1;
            let mut line = u16::abs_diff(normalized_row, normalized_line).to_string();

            match self.config.line_numbers {
                LineNumbers::RelativeNumbered => match normalized_line {
                    l if l == normalized_row => line = normalized_row.to_string(),
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
