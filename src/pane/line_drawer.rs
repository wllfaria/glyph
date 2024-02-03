use crate::config::Config;

use super::pane_dimension::PaneDimensions;
use crossterm::{
    cursor,
    style::{Color, Print, Stylize},
    QueueableCommand,
};
use std::io::{stdout, Result, Stdout};

pub trait LineDrawer {
    fn draw_lines(
        &mut self,
        dimensions: &PaneDimensions,
        total_lines: u16,
        current_line: u16,
    ) -> Result<()>;
}

#[derive(Debug)]
pub struct AbsoluteLineDrawer {
    stdout: Stdout,
    config: &'static Config,
}

impl AbsoluteLineDrawer {
    pub fn new() -> Self {
        Self {
            stdout: stdout(),
            config: Config::get(),
        }
    }
}

impl LineDrawer for AbsoluteLineDrawer {
    fn draw_lines(&mut self, dimensions: &PaneDimensions, total_lines: u16, _: u16) -> Result<()> {
        let total_lines = u16::min(dimensions.height, total_lines);

        for i in 0..total_lines {
            let line = (i + 1).to_string();
            let offset = dimensions.col + self.config.sidebar_width - line.len() as u16;

            self.stdout
                .queue(cursor::MoveTo(offset, i as u16))?
                .queue(Print(line.with(Color::DarkGrey)))?;
        }
        Ok(())
    }
}
