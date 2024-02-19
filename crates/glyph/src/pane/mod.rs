use crossterm::style::{self, Color};
use crossterm::{self, style::Print, QueueableCommand};
use std::cell::RefCell;
use std::io::{stdout, Result, Stdout};

use std::rc::Rc;

use crate::buffer::Buffer;
use crate::command::{BufferCommands, Command, CursorCommands, EditorCommands};
use crate::config::Config;
use crate::highlight::{ColorInfo, Highlight};
use crate::pane::cursor::Cursor;
use crate::pane::gutter::Gutter;
use crate::theme::{Style, Theme};

mod cursor;
mod gutter;

#[derive(Debug, Default)]
pub struct Position {
    pub row: u16,
    pub col: u16,
}

#[derive(Debug, Clone)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    c: char,
    style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            style: Theme::get().style.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Viewport {
    pub cells: Vec<Cell>,
    dimensions: PaneDimensions,
}

#[derive(Debug)]
pub struct Change<'a> {
    cell: &'a Cell,
    x: usize,
    y: usize,
}

impl Viewport {
    pub fn new(dimensions: PaneDimensions) -> Self {
        Self {
            cells: vec![Default::default(); (dimensions.width * dimensions.height) as usize],
            dimensions,
        }
    }

    pub fn set_cell(&mut self, x: usize, y: usize, c: char, style: &Style) {
        let pos = y * self.dimensions.width as usize + x;
        self.cells[pos] = Cell {
            c,
            style: style.clone(),
        };
    }

    fn set_text(&mut self, x: usize, y: usize, text: &str, style: &Style) {
        let pos = (y * self.dimensions.width as usize) + x;
        for (i, c) in text.chars().enumerate() {
            self.cells[pos + i] = Cell {
                c,
                style: style.clone(),
            }
        }
    }

    pub fn diff(&self, other: &Viewport) -> Vec<Change> {
        let mut changes = vec![];
        for (p, cell) in self.cells.iter().enumerate() {
            if *cell != other.cells[p] {
                let y = p / self.dimensions.width as usize;
                let x = p % self.dimensions.width as usize;

                changes.push(Change { x, y, cell });
            }
        }
        changes
    }
}

pub struct Pane {
    pub id: u16,
    cursor: Cursor,
    highlight: Highlight,
    scroll: Position,
    buffer: Rc<RefCell<Buffer>>,
    viewport: Viewport,
    config: &'static Config,
    gutter: Box<dyn Gutter>,
    dimensions: PaneDimensions,
    stdout: Stdout,
    theme: &'static Theme,
}

impl Pane {
    pub fn new(id: u16, buffer: Rc<RefCell<Buffer>>, dimensions: PaneDimensions) -> Self {
        Self {
            id,
            buffer,
            highlight: Highlight::new(),
            stdout: stdout(),
            viewport: Viewport::new(dimensions.clone()),
            dimensions,
            config: Config::get(),
            cursor: Cursor::new(),
            scroll: Position::default(),
            gutter: <dyn Gutter>::get_gutter(),
            theme: Theme::get(),
        }
    }

    pub fn handle(&mut self, command: Command) -> Result<()> {
        let last_viewport = self.viewport.clone();
        let mut viewport = Viewport::new(self.dimensions.clone());

        self.stdout.queue(crossterm::cursor::Hide)?;
        match command {
            Command::Editor(EditorCommands::Start) => self.initialize()?,
            Command::Cursor(_) => self.handle_cursor_command(command)?,
            Command::Buffer(_) => self.handle_buffer_command(command)?,
            Command::Pane(_) => (),
            _ => (),
        };

        self.draw_buffer(&mut viewport);
        self.draw_sidebar(&mut viewport);
        self.draw_diff(viewport.diff(&last_viewport))?;
        self.draw_cursor()?;
        self.viewport = viewport;
        self.stdout.queue(crossterm::cursor::Show)?;
        Ok(())
    }

    pub fn initialize(&mut self) -> Result<()> {
        let mut viewport = Viewport::new(self.dimensions.clone());
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
            }
            if let Some(bg) = cell.style.bg {
                self.stdout.queue(style::SetBackgroundColor(bg))?;
            }
            self.stdout.queue(Print(cell.c))?;
        }
        Ok(())
    }

    fn draw_diff(&mut self, changes: Vec<Change>) -> Result<()> {
        self.stdout.queue(crossterm::cursor::SavePosition)?;
        for change in changes {
            self.stdout
                .queue(crossterm::cursor::MoveTo(change.x as u16, change.y as u16))?;
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
                if self.cursor.row.saturating_sub(self.scroll.row) >= self.dimensions.height {
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

                for pane_line in start..self.dimensions.height {
                    self.redraw_line(pane_line, pos.row + pane_line - start);
                }

                if let (0, 1..) = (col, row) {
                    self.cursor.col = mark.size.saturating_sub(1) as u16;
                    self.cursor.absolute_position = mark.start + mark.size.saturating_sub(1);
                }
            }
            Command::Buffer(BufferCommands::NewLine) => {
                let start = (self.cursor.row - self.scroll.row).saturating_sub(1);
                for pane_line in start..self.dimensions.height {
                    self.redraw_line(pane_line, self.cursor.row + pane_line - start);
                }
            }
            _ => self.redraw_line(self.cursor.row - self.scroll.row, pos.row),
        };

        self.draw_cursor()?;

        Ok(())
    }

    fn redraw_line(&mut self, pane_line: u16, buffer_line: u16) {
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
                c if c > mark.size.saturating_sub(1) as u16 => {
                    col += mark.size.saturating_sub(1) as u16
                }
                _ => col += self.cursor.col,
            };
            self.stdout.queue(crossterm::cursor::MoveTo(
                col,
                self.cursor.row.saturating_sub(self.scroll.row),
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

    fn highlight(&mut self, buffer: &str) -> Result<Vec<ColorInfo>> {
        Ok(self.highlight.colors(buffer))
    }

    fn draw_buffer(&mut self, viewport: &mut Viewport) {
        let height = self.dimensions.height;
        let width = self.dimensions.width;
        let offset = self.dimensions.col + self.config.gutter_width;

        let default_style = self.theme.style.clone();
        let lines = self
            .buffer
            .borrow()
            .content_from(self.scroll.row as usize, self.dimensions.height as usize);
        let colors = self.highlight(&lines).unwrap();

        let mut x = offset;
        let mut y = 0;

        let mut iter = lines.chars().enumerate().peekable();
        while let Some((p, c)) = iter.next() {
            if c == '\n' {
                let fill_width = width.saturating_sub(x as u16) as usize;
                let line_fill = " ".repeat(fill_width);
                viewport.set_text(x as usize, y as usize, &line_fill, &default_style);
                x = offset;
                y += 1;
                if y > height {
                    break;
                }
                continue;
            }

            if x < width as u16 {
                if let Some(color) = colors.iter().find(|ci| ci.start <= p && ci.end > p) {
                    viewport.set_cell(x as usize, y as usize, c, &color.style);
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_test() {
        let dimensions: PaneDimensions = (10, 10).into();
        let mut vp_a = Viewport::new(dimensions.clone());
        let mut vp_b = Viewport::new(dimensions.clone());

        vp_a.set_text(0, 0, "Hello, World!", &Theme::get().style);
        vp_b.set_text(0, 0, "Goodbye, mars!", &Theme::get().style);

        let diff = vp_b.diff(&vp_a);

        println!("{diff:#?}");

        assert_eq!(1, 2);
    }
}
