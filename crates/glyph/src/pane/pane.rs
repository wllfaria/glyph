use crossterm::{self, style::Print, QueueableCommand};
use std::cell::RefCell;
use std::io::{stdout, Result, Stdout};
use std::ops::RangeBounds;
use std::rc::Rc;

use crate::buffer::Buffer;
use crate::command::{BufferCommands, Command, CursorCommands, EditorCommands};
use crate::config::Config;
use crate::pane::cursor::Cursor;
use crate::pane::line_drawer::LineDrawer;
use crate::pane::{PaneDimensions, Position};

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
        self.stdout.queue(crossterm::cursor::Hide)?;
        match command {
            Command::Editor(EditorCommands::Start) => self.initialize()?,
            Command::Cursor(_) => self.handle_cursor_command(command)?,
            Command::Buffer(_) => self.handle_buffer_command(command)?,
            Command::Pane(_) => (),
            _ => (),
        };
        self.stdout.queue(crossterm::cursor::Show)?;
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
        self.cursor.handle(&command, &mut self.buffer.borrow_mut());
        // TODO:
        // - implement buffer scroll;
        // - check if we need to redraw the sidebar
        // - check if we need to redraw the buffer
        match command {
            Command::Cursor(CursorCommands::MoveUp) => {
                self.draw_cursor()?;
            }
            Command::Cursor(CursorCommands::MoveDown) => {
                if self.cursor.row - self.scroll.row >= self.dimensions.height {
                    self.scroll.row += 1;
                    self.clear_buffer()?;
                    self.draw_buffer()?;
                    self.draw_sidebar()?
                }
                self.draw_cursor()?;
            }
            Command::Cursor(CursorCommands::MoveLeft) => {
                self.draw_cursor()?;
            }
            Command::Cursor(CursorCommands::MoveRight) => {
                self.draw_cursor()?;
            }
            _ => (),
        }
        Ok(())
    }

    fn handle_buffer_command(&mut self, command: Command) -> Result<()> {
        match command {
            Command::Buffer(BufferCommands::Type(_)) => {
                self.buffer
                    .borrow_mut()
                    .handle(&command, self.cursor.absolute_position)?;
                self.cursor.handle(&command, &mut self.buffer.borrow_mut());
                self.redraw_line(self.cursor.row - self.scroll.row, self.cursor.row + 1)?;
            }
            Command::Buffer(BufferCommands::Backspace) => match self.cursor.col {
                0 => {
                    self.cursor.handle(&command, &mut self.buffer.borrow_mut());
                    self.buffer
                        .borrow_mut()
                        .handle(&command, self.cursor.absolute_position + 1)?;
                    let start = self.cursor.row - self.scroll.row + 1;
                    self.redraw_line_range(start..=self.dimensions.height)?;
                }
                _ => {
                    self.buffer
                        .borrow_mut()
                        .handle(&command, self.cursor.absolute_position)?;
                    self.redraw_line(self.cursor.row - self.scroll.row, self.cursor.row + 1)?;
                    self.cursor.handle(&command, &mut self.buffer.borrow_mut());
                }
            },
            Command::Buffer(BufferCommands::NewLineBelow) => {
                let end = self.dimensions.height;
                self.buffer
                    .borrow_mut()
                    .handle(&command, self.cursor.absolute_position)?;
                self.cursor.handle(&command, &mut self.buffer.borrow_mut());
                let start = self.cursor.row - self.scroll.row;
                self.redraw_line_range(start - 1..end)?;
            }
            _ => self
                .buffer
                .borrow_mut()
                .handle(&command, self.cursor.absolute_position)?,
        }
        self.draw_cursor()?;
        Ok(())
    }

    fn redraw_line_range<R>(&mut self, range: R) -> Result<()>
    where
        R: RangeBounds<u16> + std::iter::Iterator<Item = u16>,
    {
        let mut i = 0;
        for line in range {
            self.redraw_line(line, self.cursor.row + i)?;
            i += 1;
        }
        Ok(())
    }

    fn redraw_line(&mut self, pane_line: u16, cursor_line: u16) -> Result<()> {
        let buffer = self.buffer.borrow();
        if let Some(mark) = buffer.marker.get_by_line(cursor_line as usize) {
            let text = buffer.line_from_mark(&mark);
            let col = self.config.sidebar_gap + self.config.sidebar_width;
            self.stdout
                .queue(crossterm::cursor::MoveTo(col, pane_line))?
                .queue(crossterm::terminal::Clear(
                    crossterm::terminal::ClearType::UntilNewLine,
                ))?
                .queue(Print(text))?;
        }

        Ok(())
    }

    fn draw_cursor(&mut self) -> Result<()> {
        if let Some(mark) = self
            .buffer
            .borrow_mut()
            .marker
            .get_by_line(self.cursor.row as usize + 1)
        {
            let mut col = self.config.sidebar_width + self.config.sidebar_gap;
            match self.cursor.col {
                c if c > mark.size.saturating_sub(1) as u16 => {
                    col += mark.size.saturating_sub(1) as u16
                }
                _ => col += self.cursor.col,
            };
            self.stdout.queue(crossterm::cursor::MoveTo(
                col,
                self.cursor.row - self.scroll.row,
            ))?;
        }
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
            self.buffer.borrow().to_string().len() as u16,
            self.cursor.row,
            self.scroll.row,
        )?;
        Ok(())
    }

    fn clear_sidebar(&mut self) -> Result<()> {
        for row in 0..self.dimensions.height {
            for col in 0..self.config.sidebar_width {
                self.stdout
                    .queue(crossterm::cursor::MoveTo(col, row))?
                    .queue(Print(" "))?;
            }
        }
        Ok(())
    }

    fn clear_buffer(&mut self) -> Result<()> {
        self.stdout.queue(crossterm::cursor::SavePosition)?;
        let offset = self.config.sidebar_width + self.config.sidebar_gap;
        for row in 0..self.dimensions.height {
            for col in offset..self.dimensions.width {
                self.stdout
                    .queue(crossterm::cursor::MoveTo(col, row))?
                    .queue(Print(" "))?;
            }
        }

        self.stdout.queue(crossterm::cursor::RestorePosition)?;

        Ok(())
    }

    fn draw_buffer(&mut self) -> Result<()> {
        self.stdout.queue(crossterm::cursor::SavePosition)?;
        let buffer = self.buffer.borrow();
        let mut lines = buffer.lines_from(self.scroll.row as usize);
        let height = self.dimensions.height;
        let offset = self.dimensions.col + self.config.sidebar_width + self.config.sidebar_gap;

        for row in 0..height {
            let line = match lines.next() {
                Some(line) => line,
                None => break,
            };
            let len = u16::min(self.dimensions.width - offset, line.len() as u16);
            let line = &line[0..len as usize];
            let line = line.iter().collect::<String>();

            self.stdout
                .queue(crossterm::cursor::MoveTo(offset, row as u16))?
                .queue(Print(line))?;
        }

        self.stdout.queue(crossterm::cursor::RestorePosition)?;

        Ok(())
    }
}
