use crossterm::{
    cursor,
    style::{Color, Print, Stylize},
    QueueableCommand,
};
use std::{
    cell::RefCell,
    io::{stdout, Result, Stdout},
    rc::Rc,
};

use crate::{buffer::Buffer, command::Command, cursor::Cursor};

#[derive(Debug)]
pub struct Pane {
    pub id: u16,
    pub cursor: Cursor,
    buffer: Rc<RefCell<Buffer>>,
    dimensions: PaneDimensions,
    pub sidebar_width: u16,
    pub sidebar_gap: u16,
    stdout: Stdout,
}

#[derive(Debug)]
pub struct PaneDimensions {
    pub row: u16,
    pub col: u16,
    pub height: u16,
    pub width: u16,
}

impl From<(u16, u16)> for PaneDimensions {
    fn from((width, height): (u16, u16)) -> Self {
        Self {
            col: 0,
            row: 0,
            width,
            height,
        }
    }
}

impl PaneDimensions {
    pub fn new(row: u16, col: u16, height: u16, width: u16) -> Self {
        PaneDimensions {
            row,
            col,
            height,
            width,
        }
    }
}

impl Pane {
    pub fn new(id: u16, buffer: Rc<RefCell<Buffer>>, dimensions: PaneDimensions) -> Self {
        Self {
            id,
            buffer,
            sidebar_width: 5,
            sidebar_gap: 1,
            stdout: stdout(),
            cursor: Cursor::new(),
            dimensions,
        }
    }

    pub fn handle_command(&self, command: Command) {
        match command {
            _ => {}
        }
    }

    pub fn resize_pane(&mut self, dimensions: PaneDimensions) {
        self.dimensions = dimensions;
    }

    pub fn render(&mut self) -> Result<()> {
        let total_lines = self.render_lines()?;
        self.render_empty_lines(0)?;
        let column = self.cursor.col + self.sidebar_width + self.sidebar_gap;
        self.stdout.queue(cursor::MoveTo(column, self.cursor.row))?;
        Ok(())
    }

    fn render_lines(&mut self) -> Result<u16> {
        let buffer_lock = self.buffer.borrow();
        let total_lines = usize::min(self.dimensions.height as usize, buffer_lock.lines.len());

        for i in 0..total_lines {
            let readable_line = i + 1_usize;
            let line_len = readable_line.to_string().len() as u16;
            let line_display = format!("{}", readable_line).with(Color::DarkGrey);
            let line_number_col = self.dimensions.col + self.sidebar_width - line_len;
            let line_col = self.dimensions.col + self.sidebar_width + self.sidebar_gap;

            self.stdout
                .queue(cursor::MoveTo(line_number_col, i as u16))?
                .queue(Print(line_display))?
                .queue(cursor::MoveTo(line_col, i as u16))?
                .queue(Print(buffer_lock.lines.get(i).unwrap()))?;
        }

        Ok(total_lines as u16)
    }

    fn render_empty_lines(&mut self, start_row: u16) -> Result<()> {
        for row in start_row..self.dimensions.height {
            self.stdout
                .queue(cursor::MoveTo(
                    self.dimensions.col + self.sidebar_width - self.sidebar_gap,
                    self.dimensions.row + row,
                ))?
                .queue(Print("~".with(Color::DarkGrey)))?;
        }
        Ok(())
    }
}
