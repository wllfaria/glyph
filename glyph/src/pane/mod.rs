use std::cell::RefCell;
use std::io::{stdout, Result, Stdout};
use std::rc::Rc;

use crossterm::style;
use crossterm::{self, style::Print, QueueableCommand};

use crate::buffer::Buffer;
use crate::config::{Action, Config, KeyAction, LineNumbers};
use crate::highlight::Highlight;
use crate::pane::cursor::Cursor;
use crate::pane::gutter::Gutter;
use crate::theme::Theme;
use crate::viewport::{Change, Viewport};

use self::gutter::absolute_line_gutter::AbsoluteLineGutter;
use self::gutter::noop_line_gutter::NoopLineDrawer;
use self::gutter::relative_line_gutter::RelativeLineDrawer;

mod cursor;
mod gutter;

#[derive(Debug, Default)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
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
    pub cursor: Cursor,
    highlight: Highlight<'a>,
    scroll: Position,
    pub buffer: Rc<RefCell<Buffer>>,
    viewport: Viewport,
    config: &'a Config,
    gutter: Box<dyn Gutter>,
    pub size: PaneSize,
    stdout: Stdout,
    theme: &'a Theme,
}

impl<'a> Pane<'a> {
    pub fn new(
        id: usize,
        buffer: Rc<RefCell<Buffer>>,
        theme: &'a Theme,
        config: &'a Config,
    ) -> Self {
        let gutter: Box<dyn Gutter> = match config.line_numbers {
            LineNumbers::Absolute => {
                Box::new(AbsoluteLineGutter::new(config.clone(), theme.clone()))
            }
            LineNumbers::Relative => {
                Box::new(RelativeLineDrawer::new(config.clone(), theme.clone()))
            }
            LineNumbers::RelativeNumbered => {
                Box::new(RelativeLineDrawer::new(config.clone(), theme.clone()))
            }
            LineNumbers::None => Box::new(NoopLineDrawer::new(config.clone(), theme.clone())),
        };

        Self {
            id,
            buffer,
            highlight: Highlight::new(theme),
            stdout: stdout(),
            size: (0, 0).into(),
            viewport: Viewport::new(0, 0),
            cursor: Cursor::new(),
            scroll: Position::default(),
            gutter,
            config,
            theme,
        }
    }

    pub fn resize(&mut self, new_size: PaneSize) {
        self.viewport.resize(new_size.width, new_size.height);
        self.size = new_size;
    }

    pub fn handle_action(&mut self, action: &KeyAction) -> Result<()> {
        let last_viewport = self.viewport.clone();
        let mut viewport = Viewport::new(self.size.width, self.size.height);

        self.stdout.queue(crossterm::cursor::Hide)?;
        match action {
            KeyAction::Simple(Action::MoveToLineStart) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::MoveToLineEnd) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::NextWord) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::MoveLeft) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::MoveDown) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::MoveUp) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::MoveRight) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::MoveToTop) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::MoveToBottom) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::InsertChar(_)) => {
                self.handle_buffer_action(action)?;
                self.handle_cursor_action(action)?;
            }
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
        let mut viewport = Viewport::new(self.size.width, self.size.height);
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

    pub fn handle_cursor_action(&mut self, action: &KeyAction) -> Result<()> {
        self.cursor.handle(action, &mut self.buffer.borrow_mut());
        match action {
            KeyAction::Simple(Action::MoveToTop) => {
                self.scroll.row = 0;
            }
            KeyAction::Simple(Action::MoveToBottom) => {
                let Position { row, .. } = self.get_cursor_readable_position();
                if self.cursor.row.saturating_sub(self.scroll.row) >= self.size.height {
                    self.scroll.row = row - self.scroll.row - self.size.height;
                }
            }
            KeyAction::Simple(Action::MoveUp) => {
                let Position { row, .. } = self.get_cursor_readable_position();
                if row.saturating_sub(self.scroll.row) == 0 {
                    self.scroll.row = self.scroll.row.saturating_sub(1);
                }
            }
            KeyAction::Simple(Action::MoveDown) => {
                if self.cursor.row.saturating_sub(self.scroll.row) >= self.size.height {
                    self.scroll.row += 1;
                }
            }
            _ => (),
        }
        self.draw_cursor()?;
        Ok(())
    }

    fn handle_buffer_action(&mut self, action: &KeyAction) -> Result<()> {
        let col = self.cursor.col;
        let row = self.cursor.row;
        let mark = {
            let buffer = self.buffer.borrow_mut();
            let mark = buffer.marker.get_by_cursor(self.cursor.absolute_position);
            mark.unwrap()
        };

        self.buffer
            .borrow_mut()
            .handle_action(action, self.cursor.absolute_position)?;

        let pos = self.get_cursor_readable_position();

        match action {
            KeyAction::Simple(Action::DeletePreviousChar) => {
                let start = self.cursor.row - self.scroll.row.saturating_sub(1);

                for pane_line in start..self.size.height {
                    self.redraw_line(pane_line, pos.row + pane_line - start);
                }

                if let (0, 1..) = (col, row) {
                    self.cursor.col = mark.size.saturating_sub(1);
                    self.cursor.absolute_position = mark.start + mark.size.saturating_sub(1);
                }
            }
            KeyAction::Simple(Action::InsertLine) => {
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
        if pane_line > len {
            return;
        }
        if let Some(mark) = buffer.marker.get_by_line(buffer_line) {
            let line = buffer.line_from_mark(&mark);
            for (x, c) in line.chars().enumerate() {
                self.viewport.set_cell(
                    x + self.config.gutter_width,
                    pane_line,
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
            .get_by_line(self.cursor.row + 1)
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
            .content_from(self.scroll.row, self.size.height);
        let colors = self.highlight.colors(&lines);

        let mut x = offset;
        let mut y = 0;

        for (p, c) in lines.chars().enumerate() {
            if c == '\n' {
                let fill_width = width.saturating_sub(x);
                let line_fill = " ".repeat(fill_width);
                viewport.set_text(x, y, &line_fill, &default_style);
                x = offset;
                y += 1;
                if y > height {
                    break;
                }
                continue;
            }

            if x < width {
                if let Some(color) = colors.iter().find(|ci| ci.start <= p && ci.end > p) {
                    viewport.set_cell(x, y, c, color.style);
                } else {
                    viewport.set_cell(x, y, c, &default_style)
                }
            }
            x += 1;
        }
    }

    pub fn get_buffer(&self) -> Rc<RefCell<Buffer>> {
        self.buffer.clone()
    }
}
