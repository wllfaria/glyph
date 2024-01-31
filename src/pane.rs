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
    command::{Command, EditorCommands},
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
            Command::Cursor(_) => self.handle_cursor_command(command)?,
            Command::Buffer(_) => self.buffer.borrow().handle(command),
            Command::Pane(_) => (),
            _ => (),
        };
        Ok(())
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.draw_sidebar()?;
        self.draw_buffer()?;
        self.draw_cursor()?;
        Ok(())
    }

    pub fn get_cursor_readable_position(&self) -> (u16, u16) {
        self.cursor.get_readable_position()
    }

    fn handle_cursor_command(&mut self, command: Command) -> Result<()> {
        self.cursor.handle(&command, &self.buffer.borrow().lines);
        self.draw_cursor()?;
        Ok(())
    }

    fn draw_cursor(&mut self) -> Result<()> {
        let col = self.cursor.col + self.sidebar_width + self.sidebar_gap;
        self.stdout.queue(cursor::MoveTo(col, self.cursor.row))?;
        Ok(())
    }

    fn draw_sidebar(&mut self) -> Result<()> {
        self.draw_line_numbers()?;
        self.draw_empty_lines()?;
        Ok(())
    }

    fn draw_line_numbers(&mut self) -> Result<()> {
        let buffer = self.buffer.borrow();
        let total_lines = usize::min(self.dimensions.height as usize, buffer.lines.len());

        for i in 0..total_lines {
            let line = (i + 1_usize).to_string();
            let offset = self.dimensions.col + self.sidebar_width - line.len() as u16;

            self.stdout
                .queue(cursor::MoveTo(offset, i as u16))?
                .queue(Print(line.with(Color::DarkGrey)))?;
        }
        Ok(())
    }

    fn draw_empty_lines(&mut self) -> Result<()> {
        let total_lines = self.buffer.borrow().lines.len() as u16;
        let offset = self.dimensions.col + self.sidebar_width - self.sidebar_gap;
        for row in total_lines..self.dimensions.height {
            self.stdout
                .queue(cursor::MoveTo(offset, self.dimensions.row + row))?
                .queue(Print("~".with(Color::DarkGrey)))?;
        }
        Ok(())
    }

    fn draw_buffer(&mut self) -> Result<()> {
        let lines = &self.buffer.borrow().lines;
        let offset = self.dimensions.col + self.sidebar_width + self.sidebar_gap;
        for row in 0..self.dimensions.height {
            let line = &lines[row as usize];
            let len = self.dimensions.width.min(line.len() as u16);
            let line = line[0..len as usize].to_string();
            self.stdout
                .queue(cursor::MoveTo(offset, row as u16))?
                .queue(Print(line))?;
        }
        Ok(())
    }
}
