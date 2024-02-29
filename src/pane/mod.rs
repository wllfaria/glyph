use std::cell::RefCell;
use std::io::{stdout, Stdout};
use std::rc::Rc;

use crossterm::{self, QueueableCommand};

use crate::buffer::Buffer;
use crate::config::{Action, Config, KeyAction, LineNumbers};
use crate::cursor::Cursor;
use crate::editor::Mode;
use crate::highlight::Highlight;
use crate::lsp::IncomingMessage;
use crate::pane::gutter::Gutter;
use crate::theme::Theme;
use crate::tui::{HoverPopup, Scrollable, TuiView};
use crate::viewport::{Cell, Viewport};

use self::gutter::absolute_line_gutter::AbsoluteLineGutter;
use self::gutter::noop_line_gutter::NoopLineDrawer;
use self::gutter::relative_line_gutter::RelativeLineDrawer;

mod gutter;

#[derive(Debug, Default)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub row: usize,
    pub col: usize,
    pub height: usize,
    pub width: usize,
}

impl From<(u16, u16)> for Rect {
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
    view: Box<dyn Scrollable + 'a>,
    pub buffer: Rc<RefCell<Buffer>>,
    viewport: Viewport,
    config: &'a Config,
    gutter: Box<dyn Gutter>,
    pub size: Rect,
    stdout: Stdout,
    theme: &'a Theme,
    popups: Vec<HoverPopup<'a>>,
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

        let size: Rect = (1, 1).into();

        Self {
            id,
            buffer,
            highlight: Highlight::new(theme),
            stdout: stdout(),
            viewport: Viewport::new(size.width, size.height),
            cursor: Cursor::new(),
            scroll: Position::default(),
            view: Box::new(TuiView::new(size.clone(), theme, config.gutter_width)),
            size,
            gutter,
            config,
            theme,
            popups: vec![],
        }
    }

    pub fn resize(&mut self, new_size: Rect) -> anyhow::Result<()> {
        let last_viewport = self.viewport.clone();
        self.size = new_size.clone();
        self.view.resize(new_size, self.config.gutter_width);

        let mut viewport = Viewport::new(self.size.width, self.size.height);
        self.draw_sidebar(&mut viewport);
        let cells = self.get_highlight();
        self.view.draw(&mut viewport, &cells, &self.scroll);
        self.view
            .render_diff(&last_viewport, &viewport, &self.theme.style)?;
        self.viewport = viewport;
        Ok(())
    }

    fn get_highlight(&mut self) -> Vec<Cell> {
        let mut result: Vec<Cell> = Vec::new();
        let mut current_byte_index = 0;
        let buffer = self
            .buffer
            .borrow()
            .content_from(self.scroll.row, self.size.height);
        let colors = self.highlight.colors(&buffer);
        let default_style = &self.theme.style;

        for c in buffer.chars() {
            let cell = match colors
                .iter()
                .find(|token| current_byte_index >= token.start && current_byte_index < token.end)
            {
                Some(token) => Cell {
                    c,
                    style: token.style.clone(),
                },
                None => Cell {
                    c,
                    style: default_style.clone(),
                },
            };
            result.push(cell);
            current_byte_index += c.len_utf8();
        }

        result
    }

    pub fn handle_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        self.popups.clear();
        match action {
            KeyAction::Simple(Action::MoveToLineStart) => {
                self.handle_cursor_action(action, mode)?
            }
            KeyAction::Simple(Action::MoveToLineEnd) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::DeletePreviousChar) => {
                self.handle_buffer_action(action, mode)?
            }
            KeyAction::Simple(Action::DeleteCurrentChar) => {
                self.handle_buffer_action(action, mode)?
            }
            KeyAction::Simple(Action::NextWord) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::MoveLeft) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::MoveDown) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::MoveUp) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::MoveRight) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::MoveToTop) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::SaveBuffer) => self.handle_buffer_action(action, mode)?,
            KeyAction::Simple(Action::MoveToBottom) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::InsertLine) => self.handle_buffer_action(action, mode)?,
            KeyAction::Simple(Action::InsertLineBelow) => {
                self.handle_buffer_action(action, mode)?
            }
            KeyAction::Simple(Action::InsertLineAbove) => {
                self.handle_buffer_action(action, mode)?
            }
            KeyAction::Simple(Action::InsertChar(_)) => self.handle_buffer_action(action, mode)?,
            _ => (),
        };

        self.redraw()?;
        self.draw_cursor(mode)?;
        Ok(())
    }

    fn redraw(&mut self) -> anyhow::Result<()> {
        self.stdout
            .queue(crossterm::cursor::Hide)?
            .queue(crossterm::cursor::SavePosition)?;
        let last_viewport = self.viewport.clone();
        let mut viewport = Viewport::new(self.size.width, self.size.height);
        self.draw_sidebar(&mut viewport);
        let cells = self.get_highlight();
        self.view.draw(&mut viewport, &cells, &mut self.scroll);
        for popup in self.popups.iter_mut() {
            popup.render(&mut viewport)?;
        }
        self.view
            .render_diff(&last_viewport, &viewport, &self.theme.style)?;
        self.viewport = viewport;
        self.stdout
            .queue(crossterm::cursor::Show)?
            .queue(crossterm::cursor::RestorePosition)?;
        Ok(())
    }

    pub fn initialize(&mut self, mode: &Mode) -> anyhow::Result<()> {
        let mut viewport = Viewport::new(self.size.width, self.size.height);
        self.draw_sidebar(&mut viewport);
        self.draw_cursor(mode)?;
        let cells = self.get_highlight();
        self.view.draw(&mut viewport, &cells, &self.scroll);
        self.view
            .render_diff(&Viewport::new(0, 0), &viewport, &self.theme.style)?;
        self.viewport = viewport;
        Ok(())
    }

    pub fn get_cursor_readable_position(&self) -> Position {
        self.cursor.get_readable_position()
    }

    pub fn handle_cursor_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        self.cursor
            .handle(action, &mut self.buffer.borrow_mut(), mode);
        self.view
            .maybe_scroll(&self.cursor, &mut self.scroll, &self.size)?;
        Ok(())
    }

    fn handle_buffer_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
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

        self.handle_cursor_action(action, mode)?;

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

    // TODO: move this to the scrollable
    fn draw_cursor(&mut self, mode: &Mode) -> anyhow::Result<()> {
        let col = {
            let buffer = self.buffer.borrow_mut();
            let mut col = 0;
            if let Some(mark) = buffer.marker.get_by_line(self.cursor.row + 1) {
                col = match mode {
                    Mode::Normal => self.cursor.col.min(mark.size.saturating_sub(2)),
                    _ => self.cursor.col.min(mark.size.saturating_sub(1)),
                };
            }
            col
        };
        self.view
            .maybe_scroll(&self.cursor, &mut self.scroll, &self.size)?;
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

    pub fn get_buffer(&self) -> Rc<RefCell<Buffer>> {
        self.buffer.clone()
    }

    pub fn handle_lsp_message(
        &mut self,
        message: (IncomingMessage, Option<String>),
    ) -> anyhow::Result<()> {
        if let Some(method) = message.1 {
            match method.as_str() {
                "textDocument/hover" => {
                    let message = message.0;
                    match message {
                        IncomingMessage::Message(message) => {
                            let result = match message.result {
                                serde_json::Value::Array(ref arr) => arr[0].as_object().unwrap(),
                                serde_json::Value::Object(ref obj) => obj,
                                _ => return Ok(()),
                            };
                            if let Some(contents) = result.get("contents") {
                                if let Some(contents) = contents.as_object() {
                                    if let Some(serde_json::Value::String(value)) =
                                        contents.get("value")
                                    {
                                        let offset = self.config.gutter_width;
                                        let popup = HoverPopup::new(
                                            self.cursor.col - self.scroll.col + offset,
                                            self.cursor.row - self.scroll.row,
                                            self.theme,
                                            value.into(),
                                        );
                                        self.popups.push(popup);
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        };
        self.redraw()?;
        Ok(())
    }
}
