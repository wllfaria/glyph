use std::cell::RefCell;
use std::io::{stdout, Result, Stdout};
use std::rc::Rc;

use crossterm::style;
use crossterm::{self, style::Print, QueueableCommand};

use crate::buffer::Buffer;
use crate::config::{Action, Config, KeyAction, LineNumbers};
use crate::highlight::Highlight;
use crate::lsp::IncomingMessage;
use crate::pane::cursor::Cursor;
use crate::pane::gutter::Gutter;
use crate::theme::Theme;
use crate::viewport::{Cell, Change, Viewport};

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
    popups: Vec<Pane<'a>>,
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
            popups: vec![],
        }
    }

    pub fn resize(&mut self, new_size: PaneSize) {
        self.viewport.resize(new_size.width, new_size.height);
        self.size = new_size;
    }

    pub fn handle_action(&mut self, action: &KeyAction) -> anyhow::Result<()> {
        let last_viewport = self.viewport.clone();
        let mut viewport = Viewport::new(self.size.width, self.size.height);

        self.stdout.queue(crossterm::cursor::Hide)?;
        match action {
            KeyAction::Simple(Action::MoveToLineStart) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::MoveToLineEnd) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::DeletePreviousChar) => self.handle_buffer_action(action)?,
            KeyAction::Simple(Action::DeleteCurrentChar) => self.handle_buffer_action(action)?,
            KeyAction::Simple(Action::NextWord) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::MoveLeft) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::MoveDown) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::MoveUp) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::MoveRight) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::MoveToTop) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::SaveBuffer) => self.handle_buffer_action(action)?,
            KeyAction::Simple(Action::MoveToBottom) => self.handle_cursor_action(action)?,
            KeyAction::Simple(Action::InsertLine) => self.handle_buffer_action(action)?,
            KeyAction::Simple(Action::InsertLineBelow) => self.handle_buffer_action(action)?,
            KeyAction::Simple(Action::InsertLineAbove) => self.handle_buffer_action(action)?,
            KeyAction::Simple(Action::InsertChar(_)) => self.handle_buffer_action(action)?,
            _ => (),
        };

        self.draw_sidebar(&mut viewport);
        self.draw_cursor()?;
        self.draw_buffer(&mut viewport);
        self.draw_diff(viewport.diff(&last_viewport))?;
        self.viewport = viewport;
        self.stdout.queue(crossterm::cursor::Show)?;
        Ok(())
    }

    // TODO: I have to make this nicer, and also rendering the messages is somehow breaking the
    // entire view, so I should account for that
    //
    // Maybe the window should be responsible for making the popup and handing to the pane. this is
    // just a WIP
    pub fn handle_lsp_message(&mut self, message: (IncomingMessage, Option<String>)) {
        if let Some(method) = message.1 {
            match method.as_str() {
                "textDocument/hover" => match message.0 {
                    IncomingMessage::Message(msg) => {
                        let result = match msg.result {
                            serde_json::Value::Array(ref arr) => arr[0].as_object().unwrap(),
                            serde_json::Value::Object(ref obj) => obj,
                            _ => return,
                        };
                        if let Some(contents) = result.get("contents") {
                            if let Some(contents) = contents.as_object() {
                                if let Some(serde_json::Value::String(value)) =
                                    contents.get("value")
                                {
                                    let buf = Buffer::from_string(1, value, 0);
                                    let mut pane = Pane::new(
                                        1,
                                        Rc::new(RefCell::new(buf)),
                                        self.theme,
                                        self.config,
                                    );
                                    pane.resize((40, 10).into());
                                    pane.size.row = self.cursor.row;
                                    pane.size.col = self.cursor.col;
                                    self.popups.push(pane);
                                    tracing::debug!("Opening lsp hover popup");
                                    _ = self.draw_popups();
                                }
                            }
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }

    fn draw_popups(&mut self) -> anyhow::Result<()> {
        for popup in self.popups.iter_mut() {
            popup.initialize()?;
        }

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
        self.stdout.queue(crossterm::cursor::MoveTo(
            self.size.col as u16,
            self.size.row as u16,
        ))?;
        tracing::debug!("amount of cells {:?}", &viewport.cells.len());
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
                (self.size.col + change.col) as u16,
                (self.size.row + change.row) as u16,
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
        self.maybe_scroll(self.cursor.col, self.cursor.row);
        Ok(())
    }

    fn maybe_scroll(&mut self, col: usize, row: usize) {
        let height = self.size.height;
        let width = self.size.width - self.config.gutter_width;
        match (col, row) {
            // Should scroll down
            (_, y) if (y + 1).saturating_sub(self.scroll.row) >= height => {
                self.scroll.row = y + 1 - height;
            }
            // Should scroll up
            (_, y) if (y + 1).saturating_sub(self.scroll.row) == 0 => {
                self.scroll.row -= self.scroll.row - y;
            }
            // Should scroll right
            (x, _) if x.saturating_sub(self.scroll.col) >= width => {
                self.scroll.col = x + 1 - width;
            }
            // Should scroll left
            (x, _) if (x + 1).saturating_sub(self.scroll.col) == 0 => {
                self.scroll.col -= self.scroll.col - x;
            }
            _ => (),
        }
    }

    fn handle_buffer_action(&mut self, action: &KeyAction) -> anyhow::Result<()> {
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

        self.handle_cursor_action(action)?;

        match action {
            KeyAction::Simple(Action::DeletePreviousChar) => {
                if let (0, 1..) = (col, row) {
                    self.cursor.col = mark.size.saturating_sub(1);
                    self.cursor.absolute_position = mark.start + mark.size.saturating_sub(1);
                }
            }
            _ => (),
        };

        Ok(())
    }

    fn draw_cursor(&mut self) -> Result<()> {
        let col = {
            let buffer = self.buffer.borrow_mut();
            let mut col = 0;
            if let Some(mark) = buffer.marker.get_by_line(self.cursor.row + 1) {
                col += match self.cursor.col {
                    c if c > mark.size.saturating_sub(1) => mark.size.saturating_sub(1),
                    _ => self.cursor.col,
                };
            }
            col
        };
        tracing::debug!("col: {}, row: {}", col, self.cursor.row);
        self.maybe_scroll(col, self.cursor.row);
        self.stdout.queue(crossterm::cursor::MoveTo(
            col.saturating_sub(self.scroll.col) as u16 + self.config.gutter_width as u16,
            self.cursor.row.saturating_sub(self.scroll.row) as u16,
        ))?;
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
        let offset = self.size.col + self.config.gutter_width;

        let default_style = self.theme.style.clone();
        let content = self
            .buffer
            .borrow()
            .content_from(self.scroll.row, self.size.height);
        let colors = self.highlight.colors(&content);

        let mut abs_pos = 0;
        // TODO: I don't really like how I'm handling this, but this works for now
        for (y, line) in content.lines().enumerate() {
            if y > 0 {
                abs_pos += 1;
            }
            for (x, c) in line.chars().enumerate() {
                if x >= self.scroll.col
                    && x - self.scroll.col < self.size.width - self.config.gutter_width
                {
                    if let Some(color) = colors
                        .iter()
                        .find(|ci| ci.start <= abs_pos && ci.end > abs_pos)
                    {
                        viewport.set_cell((x - self.scroll.col) + offset, y, c, color.style);
                    } else {
                        viewport.set_cell((x - self.scroll.col) + offset, y, c, &default_style);
                    }
                }
                abs_pos += 1;
            }
        }
    }

    pub fn get_buffer(&self) -> Rc<RefCell<Buffer>> {
        self.buffer.clone()
    }
}
