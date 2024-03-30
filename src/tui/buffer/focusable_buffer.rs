use std::io::stdout;

use crate::{
    config::{Action, KeyAction},
    cursor::Cursor,
    editor::Mode,
    pane::Position,
    tui::{buffer::Buffer, rect::Rect, Focusable, Renderable},
};

pub struct FocusableBuffer<'a> {
    buffer: Buffer<'a>,
    cursor: Cursor,
}

impl<'a> FocusableBuffer<'a> {
    pub fn new(buffer: Buffer<'a>, cursor: Cursor) -> Self {
        Self { buffer, cursor }
    }

    fn handle_cursor_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        self.cursor
            .handle(action, &mut self.buffer.text_object.borrow_mut(), mode);

        tracing::trace!(
            "[FocusableBuffer] cursor: {:?} area: {:?}",
            self.cursor,
            self.buffer.area
        );

        match (self.cursor.col, self.cursor.row) {
            (x, _) if x as u16 >= self.buffer.area.width => {
                self.cursor.col = self.buffer.area.width.saturating_sub(1) as usize;
            }
            (_, y) if y as u16 >= self.buffer.area.height => {
                self.cursor.row = self.buffer.area.height.saturating_sub(1) as usize;
            }
            _ => (),
        }

        Ok(())
    }

    fn handle_text_object_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        let col = self.cursor.col;
        let row = self.cursor.row;
        let mark = {
            let buffer = self.buffer.text_object.borrow_mut();
            let mark = buffer.marker.get_by_cursor(self.cursor.absolute_position);
            mark.unwrap()
        };

        self.buffer
            .text_object
            .borrow_mut()
            .handle_action(action, self.cursor.absolute_position)?;

        self.cursor
            .handle(action, &mut self.buffer.text_object.borrow_mut(), mode);

        if let KeyAction::Simple(Action::DeletePreviousChar) = action {
            if let (0, 1..) = (col, row) {
                self.cursor.col = mark.size.saturating_sub(1);
                self.cursor.absolute_position = mark.start + mark.size.saturating_sub(1);
            }
        };

        Ok(())
    }
}

impl Renderable<'_> for FocusableBuffer<'_> {
    fn render(&mut self, frame: &mut crate::frame::Frame) -> anyhow::Result<()> {
        self.buffer.render(frame)?;

        Ok(())
    }

    fn resize(&mut self, new_area: Rect) -> anyhow::Result<()> {
        self.buffer.resize(new_area)?;

        Ok(())
    }
}

impl Focusable<'_> for FocusableBuffer<'_> {
    fn render_cursor(&self, mode: &Mode) -> anyhow::Result<()> {
        let gutter_size = self.buffer.config.gutter_width;

        let col = {
            let mut col = 0;
            let marker = &self.buffer.text_object.borrow().marker;
            let Position { row, .. } = self.cursor.get_readable_position();

            if let Some(mark) = marker.get_by_line(row) {
                col = match mode {
                    Mode::Normal => self.cursor.col.min(mark.size.saturating_sub(2)),
                    _ => self.cursor.col.min(mark.size.saturating_sub(1)),
                }
            }

            col
        };

        crossterm::queue!(
            stdout(),
            crossterm::cursor::MoveTo(col as u16 + gutter_size as u16, self.cursor.row as u16,)
        )?;

        Ok(())
    }

    fn handle_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        tracing::debug!("[FocusableBuffer] handling action: {:?}", action);
        match action {
            KeyAction::Simple(Action::MoveToLineStart) => {
                self.handle_cursor_action(action, mode)?;
            }
            KeyAction::Simple(Action::MoveToLineEnd) => {
                self.handle_cursor_action(action, mode)?;
            }
            KeyAction::Simple(Action::NextWord) => {
                self.handle_cursor_action(action, mode)?;
            }
            KeyAction::Simple(Action::MoveLeft) => {
                self.handle_cursor_action(action, mode)?;
            }
            KeyAction::Simple(Action::MoveDown) => {
                self.handle_cursor_action(action, mode)?;
            }
            KeyAction::Simple(Action::MoveUp) => {
                self.handle_cursor_action(action, mode)?;
            }
            KeyAction::Simple(Action::MoveRight) => {
                self.handle_cursor_action(action, mode)?;
            }
            KeyAction::Simple(Action::MoveToTop) => {
                self.handle_cursor_action(action, mode)?;
            }
            KeyAction::Simple(Action::SaveBuffer) => {
                self.handle_text_object_action(action, mode)?
            }
            KeyAction::Simple(Action::InsertLine) => {
                self.handle_text_object_action(action, mode)?
            }
            KeyAction::Simple(Action::InsertLineBelow) => {
                self.handle_text_object_action(action, mode)?
            }
            KeyAction::Simple(Action::DeletePreviousChar) => {
                self.handle_text_object_action(action, mode)?
            }
            KeyAction::Simple(Action::DeleteCurrentChar) => {
                self.handle_text_object_action(action, mode)?
            }
            KeyAction::Simple(Action::InsertLineAbove) => {
                self.handle_text_object_action(action, mode)?
            }
            KeyAction::Simple(Action::InsertChar(_)) => {
                self.handle_text_object_action(action, mode)?
            }
            _ => (),
        };

        Ok(())
    }
}
