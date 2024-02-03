use crossterm::{cursor, style::Print, QueueableCommand};
use std::cell::RefCell;
use std::io::{stdout, Result, Stdout};
use std::rc::Rc;

use crate::buffer::Buffer;
use crate::command::{Command, EditorCommands};
use crate::config::{Config, LineNumbers};
use crate::cursor::Cursor;
use crate::pane::{AbsoluteLineDrawer, LineDrawer, PaneDimensions};

pub struct Pane {
    pub id: u16,
    pub cursor: Cursor,
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
            line_drawer: Pane::get_line_drawer(),
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
        let col = self.cursor.col + self.config.sidebar_width + self.config.sidebar_gap;
        self.stdout.queue(cursor::MoveTo(col, self.cursor.row))?;
        Ok(())
    }

    fn draw_sidebar(&mut self) -> Result<()> {
        self.line_drawer.draw_lines(
            &self.dimensions,
            self.buffer.borrow().lines.len() as u16,
            self.cursor.row,
        )?;
        Ok(())
    }

    fn draw_buffer(&mut self) -> Result<()> {
        let lines = &self.buffer.borrow().lines;
        let offset = self.dimensions.col + self.config.sidebar_width + self.config.sidebar_gap;
        let total_lines = self.dimensions.height.min(lines.len() as u16);
        for row in 0..total_lines {
            let line = &lines[row as usize];
            let len = self.dimensions.width.min(line.len() as u16);
            let line = line[0..len as usize].to_string();
            self.stdout
                .queue(cursor::MoveTo(offset, row as u16))?
                .queue(Print(line))?;
        }
        Ok(())
    }

    fn get_line_drawer() -> Box<dyn LineDrawer> {
        let config = Config::get();
        // TODO: implement the others line drwaers
        match config.line_numbers {
            LineNumbers::Absolute => Box::new(AbsoluteLineDrawer::new()),
            LineNumbers::Relative => Box::new(AbsoluteLineDrawer::new()),
            LineNumbers::RelativeNumbered => Box::new(AbsoluteLineDrawer::new()),
            LineNumbers::None => Box::new(AbsoluteLineDrawer::new()),
        }
    }
}

impl std::fmt::Debug for Pane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pane")
            .field("id", &self.id)
            .field("cursor", &self.cursor)
            .field("buffer", &self.buffer)
            .field("config", &self.config)
            .field("dimensions", &self.dimensions)
            .finish()
    }
}
