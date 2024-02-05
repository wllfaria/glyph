use crossterm::{cursor, style::Print, QueueableCommand};
use std::cell::RefCell;
use std::io::{stdout, Result, Stdout};
use std::rc::Rc;

use crate::buffer::Buffer;
use crate::command::{Command, CursorCommands, EditorCommands};
use crate::config::Config;
use crate::cursor::Cursor;
use crate::pane::{LineDrawer, PaneDimensions, Position};

#[derive(Debug)]
pub struct Pane {
    pub id: u16,
    pub cursor: Cursor,
    scroll: Position,
    buffer: Rc<RefCell<Buffer>>,
    config: &'static Config,
    line_drawer: Box<dyn LineDrawer>,
    dimensions: PaneDimensions,
    stdout: Stdout,
}

impl Pane {
    pub fn new(id: u16, buffer: Rc<RefCell<Buffer>>, dimensions: PaneDimensions) -> Self {
        Self {
            id,
            buffer,
            dimensions,
            stdout: stdout(),
            config: Config::get(),
            cursor: Cursor::new(),
            scroll: Position::default(),
            line_drawer: <dyn LineDrawer>::get_line_drawer(),
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
        match command {
            Command::Cursor(CursorCommands::MoveUp) => {
                if self.cursor.position.row < self.scroll.row {
                    self.scroll.row = self.cursor.position.row;
                    self.clear_buffer()?;
                    self.draw_buffer()?;
                }
                self.maybe_redraw_sidebar()?;
            }
            Command::Cursor(CursorCommands::MoveDown) => {
                if self.cursor.position.row == self.scroll.row + self.dimensions.height {
                    self.scroll.row += 1;
                    self.clear_buffer()?;
                    self.draw_buffer()?;
                }
                self.maybe_redraw_sidebar()?;
            }
            _ => (),
        }
        self.draw_cursor()?;
        Ok(())
    }

    fn draw_cursor(&mut self) -> Result<()> {
        let col = self.cursor.position.col + self.config.sidebar_width + self.config.sidebar_gap;
        self.stdout.queue(cursor::MoveTo(
            col,
            self.cursor.position.row - self.scroll.row,
        ))?;
        Ok(())
    }

    fn maybe_redraw_sidebar(&mut self) -> Result<()> {
        self.draw_sidebar()?;
        Ok(())
    }

    fn draw_sidebar(&mut self) -> Result<()> {
        self.clear_sidebar()?;
        self.line_drawer.draw_lines(
            &self.dimensions,
            self.buffer.borrow().lines.len() as u16,
            self.cursor.position.row,
            self.scroll.row,
        )?;
        Ok(())
    }

    fn clear_sidebar(&mut self) -> Result<()> {
        for row in 0..self.dimensions.height {
            for col in 0..self.config.sidebar_width {
                self.stdout
                    .queue(cursor::MoveTo(col, row))?
                    .queue(Print(" "))?;
            }
        }
        Ok(())
    }

    fn clear_buffer(&mut self) -> Result<()> {
        let offset = self.config.sidebar_width + self.config.sidebar_gap;
        for row in 0..self.dimensions.height {
            for col in offset..self.dimensions.width {
                self.stdout
                    .queue(cursor::MoveTo(col, row))?
                    .queue(Print(" "))?;
            }
        }
        Ok(())
    }

    fn draw_buffer(&mut self) -> Result<()> {
        let lines = &self.buffer.borrow().lines[self.scroll.row as usize..];
        let offset = self.dimensions.col + self.config.sidebar_width + self.config.sidebar_gap;
        let total_lines = self.dimensions.height.min(lines.len() as u16);
        for row in 0..total_lines {
            let line = &lines[row as usize];
            let len = u16::min(self.dimensions.width - offset, line.len() as u16);
            let line = line[0..len as usize].to_string();
            self.stdout
                .queue(cursor::MoveTo(offset, row as u16))?
                .queue(Print(line))?;
        }
        Ok(())
    }
}
