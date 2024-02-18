use crate::config::Config;
use crate::pane::PaneDimensions;
use crate::theme::Theme;
use crossterm::style::{self, Color, Print, Stylize};
use crossterm::{cursor, QueueableCommand};
use std::io;

use crate::pane::gutter::Gutter;

#[derive(Debug)]
pub struct AbsoluteLineGutter {
    stdout: io::Stdout,
    config: &'static Config,
}

impl AbsoluteLineGutter {
    pub fn new() -> Self {
        Self {
            stdout: io::stdout(),
            config: Config::get(),
        }
    }
}

impl Gutter for AbsoluteLineGutter {
    fn draw(
        &mut self,
        dimensions: &PaneDimensions,
        total_lines: u16,
        _: u16,
        scroll_row: u16,
    ) -> io::Result<()> {
        let total_lines = u16::min(dimensions.height, total_lines);
        let mut scroll_row = scroll_row;
        let theme = Theme::get();

        for i in 0..total_lines {
            scroll_row += 1;
            let mut line = scroll_row.to_string();
            line = " ".repeat(self.config.gutter_width as usize - 1 - line.len()) + &line;
            line.push_str(" ");

            self.stdout
                .queue(cursor::MoveTo(dimensions.col, i))?
                .queue(style::SetBackgroundColor(theme.style.bg.unwrap()))?
                .queue(Print(line.with(Color::DarkGrey)))?;
        }

        if total_lines < dimensions.height {
            for i in total_lines..dimensions.height {
                let mut line = " ".repeat(self.config.gutter_width as usize - 2);
                line.push(self.config.empty_line_char);
                line.push(' ');
                self.stdout
                    .queue(cursor::MoveTo(dimensions.col, i))?
                    .queue(style::SetBackgroundColor(theme.style.bg.unwrap()))?
                    .queue(Print(line))?;
            }
        }

        Ok(())
    }
}
