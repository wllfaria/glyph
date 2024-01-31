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

use crate::{
    buffer::Buffer,
    command::{Command, CursorCommands, EditorCommands, PaneCommands},
    cursor::Cursor,
};

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

    pub fn handle(&mut self, command: Command) -> Result<()> {
        match command {
            Command::Editor(EditorCommands::Start) => self.initialize()?,
            Command::Cursor(_) => self.handle_cursor(command)?,
            Command::Buffer(_) => self.buffer.borrow().handle(command),
            Command::Pane(_) => (),
            _ => (),
        };
        Ok(())
    }

    fn handle_cursor(&mut self, command: Command) -> Result<()> {
        self.cursor.handle(&command, &self.buffer.borrow().lines);
        self.draw_cursor()?;
        Ok(())
    }

    fn draw_cursor(&mut self) -> Result<()> {
        let col = self.cursor.col + self.sidebar_width + self.sidebar_gap;
        self.stdout.queue(cursor::MoveTo(col, self.cursor.row))?;
        Ok(())
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.draw_sidebar()?;
        self.draw_cursor()?;
        Ok(())
    }

    fn draw_lines(&mut self) -> Result<u16> {
        let buffer = self.buffer.borrow();
        let total_lines = usize::min(self.dimensions.height as usize, buffer.lines.len());

        for i in 0..total_lines {
            let line = (i + 1_usize).to_string();
            let offset = self.dimensions.col + self.sidebar_width - line.len() as u16;

            self.stdout
                .queue(cursor::MoveTo(offset, i as u16))?
                .queue(Print(line.with(Color::DarkGrey)))?;
        }

        Ok(total_lines as u16)
    }

    fn draw_sidebar(&mut self) -> Result<()> {
        let total_lines = self.draw_lines()?;
        for row in total_lines..self.dimensions.height {
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
