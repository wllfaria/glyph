use std::{
    io::{stdout, Result, Stdout, Write},
    sync::{Arc, Mutex},
};

use crossterm::{
    cursor,
    style::{Color, Print, Stylize},
    terminal, QueueableCommand,
};

use crate::buffer::Buffer;

#[derive(Debug)]
pub struct Pane {
    pub id: u16,
    pub buffer: Arc<Mutex<Buffer>>,
    pub row: u16,
    pub col: u16,
    pub height: u16,
    pub width: u16,
    stdout: Stdout,
}

impl Pane {
    pub fn new(id: u16, buffer: Arc<Mutex<Buffer>>) -> Self {
        Self {
            id,
            row: 0,
            col: 0,
            height: 0,
            width: 0,
            buffer,
            stdout: stdout(),
        }
    }

    pub fn set_pane_position(&mut self, row: u16, col: u16, height: u16, width: u16) {
        self.row = row;
        self.col = col;
        self.height = height;
        self.width = width;
    }

    pub fn render(&mut self) -> Result<()> {
        self.render_empty_lines()?;
        Ok(())
    }

    fn render_empty_lines(&mut self) -> Result<()> {
        for row in 0..self.height {
            self.stdout
                .queue(cursor::MoveTo(self.col, self.row + row))?
                .queue(Print(
                    format!("{}-{}", self.col, self.width).with(Color::DarkGrey),
                ))?;
        }
        self.stdout.queue(cursor::MoveTo(self.col, self.row))?;
        Ok(())
    }
}
