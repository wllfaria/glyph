use std::cell::RefCell;
use std::io::{stdout, Result, Stdout};
use std::rc::Rc;

use crossterm::style;
use crossterm::{self, style::Print, QueueableCommand};

use crate::buffer::Buffer;
use crate::command::{BufferCommands, Command, CursorCommands, EditorCommands};
use crate::config::Config;
use crate::highlight::Highlight;
use crate::lsp::LspClient;
use crate::pane::cursor::Cursor;
use crate::pane::gutter::Gutter;
use crate::theme::Theme;
use crate::viewport::{Change, Viewport};

mod cursor;
mod gutter;

#[derive(Debug, Default)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct PaneSize {
    pub row: usize,
    pub col: usize,
    pub height: usize,
    pub width: usize,
}

impl From<(u16, u16)> for PaneSize {
    fn from((width, height): (u16, u16)) -> Self {
        Self {
            col: 0,
            row: 0,
            width: width as usize,
            height: height as usize,
        }
    }
}

pub struct Pane<'a> {
    pub id: usize,
    cursor: Cursor,
    highlight: Highlight,
    scroll: Position,
    buffer: Rc<RefCell<Buffer>>,
    viewport: Viewport,
    config: &'static Config,
    gutter: Box<dyn Gutter>,
    size: PaneSize,
    stdout: Stdout,
    theme: &'static Theme,
    lsp: &'a LspClient,
}

impl<'a> Pane<'a> {
    pub fn new(id: usize, buffer: Rc<RefCell<Buffer>>, lsp: &'a LspClient) -> Self {
        Self {
            id,
            buffer,
            highlight: Highlight::new(),
            stdout: stdout(),
            size: (0, 0).into(),
            viewport: Viewport::new(0, 0),
            config: Config::get(),
            cursor: Cursor::new(),
            scroll: Position::default(),
            gutter: <dyn Gutter>::get_gutter(),
            theme: Theme::get(),
            lsp,
        }
    }

    pub fn resize(&mut self, new_size: PaneSize) {
        self.viewport.resize(new_size.width, new_size.height);
        self.size = new_size;
    }

    pub fn handle(&mut self, command: Command) -> Result<()> {
        let last_viewport = self.viewport.clone();
        let mut viewport = Viewport::new(self.size.width as usize, self.size.height as usize);

        self.stdout.queue(crossterm::cursor::Hide)?;
        match command {
            Command::Editor(EditorCommands::Start) => self.initialize()?,
            Command::Cursor(_) => self.handle_cursor_command(command)?,
            Command::Buffer(_) => self.handle_buffer_command(command)?,
            Command::Pane(_) => (),
            _ => (),
        };

        self.draw_sidebar(&mut viewport);
        self.draw_buffer(&mut viewport);
        self.draw_diff(viewport.diff(&last_viewport))?;
        self.draw_cursor()?;
        self.viewport = viewport;
        self.stdout.queue(crossterm::cursor::Show)?;
        Ok(())
    }

    pub fn initialize(&mut self) -> Result<()> {
        let mut viewport = Viewport::new(self.size.width as usize, self.size.height as usize);
        self.draw_sidebar(&mut viewport);
        self.draw_buffer(&mut viewport);
        self.draw(&mut viewport)?;
        self.draw_cursor()?;
        self.viewport = viewport;
        Ok(())
    }

    fn draw(&mut self, viewport: &mut Viewport) -> Result<()> {
        self.stdout.queue(crossterm::cursor::MoveTo(0, 0))?;
        for cell in &viewport.cells {
            if let Some(fg) = cell.style.fg {
                self.stdout.queue(style::SetForegroundColor(fg))?;
            } else {
                self.stdout
                    .queue(style::SetForegroundColor(self.theme.style.fg.unwrap()))?;
            }
            if let Some(bg) = cell.style.bg {
                self.stdout.queue(style::SetBackgroundColor(bg))?;
            } else {
                self.stdout
                    .queue(style::SetBackgroundColor(self.theme.style.bg.unwrap()))?;
            }
            self.stdout.queue(Print(cell.c))?;
        }
        Ok(())
    }

    fn draw_diff(&mut self, changes: Vec<Change>) -> Result<()> {
        self.stdout.queue(crossterm::cursor::SavePosition)?;
        for change in changes {
            self.stdout.queue(crossterm::cursor::MoveTo(
                change.col as u16,
                change.row as u16,
            ))?;
            if let Some(bg) = change.cell.style.bg {
                self.stdout.queue(style::SetBackgroundColor(bg))?;
            } else {
                self.stdout
                    .queue(style::SetBackgroundColor(self.theme.style.bg.unwrap()))?;
            }
            if let Some(fg) = change.cell.style.fg {
                self.stdout.queue(style::SetForegroundColor(fg))?;
            } else {
                self.stdout
                    .queue(style::SetForegroundColor(self.theme.style.fg.unwrap()))?;
            }
            self.stdout.queue(style::Print(change.cell.c))?;
        }
        self.stdout.queue(crossterm::cursor::RestorePosition)?;
        Ok(())
    }

    pub fn get_cursor_readable_position(&self) -> Position {
        self.cursor.get_readable_position()
    }

    fn handle_cursor_command(&mut self, command: Command) -> Result<()> {
        self.cursor.handle(&command, &mut self.buffer.borrow_mut());
        match command {
            Command::Cursor(CursorCommands::MoveUp) => {
                let Position { row, .. } = self.get_cursor_readable_position();
                if row.saturating_sub(self.scroll.row) == 0 {
                    self.scroll.row = self.scroll.row.saturating_sub(1);
                }
            }
            Command::Cursor(CursorCommands::MoveDown) => {
                if self.cursor.row.saturating_sub(self.scroll.row) >= self.size.height {
                    self.scroll.row += 1;
                }
            }
            _ => (),
        }
        self.draw_cursor()?;
        Ok(())
    }

    fn handle_buffer_command(&mut self, command: Command) -> Result<()> {
        let col = self.cursor.col;
        let row = self.cursor.row;
        let mark = {
            let buffer = self.buffer.borrow_mut();
            let mark = buffer.marker.get_by_line(self.cursor.row as usize);
            mark.unwrap()
        };

        self.buffer
            .borrow_mut()
            .handle(&command, self.cursor.absolute_position)?;
        self.cursor.handle(&command, &mut self.buffer.borrow_mut());

        let pos = self.get_cursor_readable_position();

        match command {
            Command::Buffer(BufferCommands::Backspace) => {
                let start = self.cursor.row - self.scroll.row.saturating_sub(1);

                for pane_line in start..self.size.height {
                    self.redraw_line(pane_line, pos.row + pane_line - start);
                }

                if let (0, 1..) = (col, row) {
                    self.cursor.col = mark.size.saturating_sub(1);
                    self.cursor.absolute_position = mark.start + mark.size.saturating_sub(1);
                }
            }
            Command::Buffer(BufferCommands::NewLine) => {
                let start = (self.cursor.row - self.scroll.row).saturating_sub(1);
                for pane_line in start..self.size.height {
                    self.redraw_line(pane_line, self.cursor.row + pane_line - start);
                }
            }
            _ => self.redraw_line(self.cursor.row - self.scroll.row, pos.row),
        };

        self.draw_cursor()?;

        Ok(())
    }

    fn redraw_line(&mut self, pane_line: usize, buffer_line: usize) {
        let buffer = self.buffer.borrow();
        let len = buffer.marker.len();
        if pane_line as usize > len {
            return;
        }
        if let Some(mark) = buffer.marker.get_by_line(buffer_line as usize) {
            let line = buffer.line_from_mark(&mark);
            for (x, c) in line.chars().enumerate() {
                self.viewport.set_cell(
                    x + self.config.gutter_width as usize,
                    pane_line as usize,
                    c,
                    &self.theme.style,
                )
            }
        }
    }

    fn draw_cursor(&mut self) -> Result<()> {
        if let Some(mark) = self
            .buffer
            .borrow_mut()
            .marker
            .get_by_line(self.cursor.row as usize + 1)
        {
            let mut col = self.config.gutter_width;
            match self.cursor.col {
                c if c > mark.size.saturating_sub(1) => col += mark.size.saturating_sub(1),
                _ => col += self.cursor.col,
            };
            self.stdout.queue(crossterm::cursor::MoveTo(
                col as u16,
                self.cursor.row.saturating_sub(self.scroll.row) as u16,
            ))?;
        }
        Ok(())
    }

    fn draw_sidebar(&mut self, viewport: &mut Viewport) {
        self.gutter.draw(
            viewport,
            self.buffer.borrow().marker.len(),
            self.cursor.row,
            self.scroll.row,
        );
    }

    fn draw_buffer(&mut self, viewport: &mut Viewport) {
        let height = self.size.height;
        let width = self.size.width;
        let offset = self.size.col + self.config.gutter_width;

        let default_style = self.theme.style.clone();
        let lines = self
            .buffer
            .borrow()
            .content_from(self.scroll.row as usize, self.size.height as usize);
        let colors = self.highlight.colors(&lines);

        let mut x = offset;
        let mut y = 0;

        for (p, c) in lines.chars().enumerate() {
            if c == '\n' {
                let fill_width = width.saturating_sub(x) as usize;
                let line_fill = " ".repeat(fill_width);
                viewport.set_text(x as usize, y as usize, &line_fill, &default_style);
                x = offset;
                y += 1;
                if y > height {
                    break;
                }
                continue;
            }

            if x < width {
                if let Some(color) = colors.iter().find(|ci| ci.start <= p && ci.end > p) {
                    viewport.set_cell(x as usize, y as usize, c, color.style);
                } else {
                    viewport.set_cell(x as usize, y as usize, c, &default_style)
                }
            }
            x += 1;
        }
    }

    pub fn get_buffer(&self) -> Rc<RefCell<Buffer>> {
        self.buffer.clone()
    }
}
