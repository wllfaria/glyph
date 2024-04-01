use std::{cell::RefCell, io::stdout, rc::Rc};

use crate::{
    buffer::TextObject,
    config::{Action, Config, KeyAction},
    cursor::Cursor,
    editor::Mode,
    frame::cell::Cell,
    highlight::Highlight,
    theme::Theme,
    tui::{
        gutter::{get_gutter, GutterKind},
        position::Position,
        rect::Rect,
        Focusable, Renderable, Scrollable,
    },
};

use super::render_within_bounds;

#[derive(Debug)]
pub struct FocusableBuffer<'a> {
    _id: usize,
    text_object: Rc<RefCell<TextObject>>,
    area: Rect,
    config: &'a Config,
    theme: &'a Theme,
    gutter: GutterKind<'a>,
    highlighter: Highlight<'a>,
    pub cursor: Cursor,
    scroll: Position,
    is_scrollable: bool,
}

impl<'a> FocusableBuffer<'a> {
    pub fn new(
        _id: usize,
        text_object: Rc<RefCell<TextObject>>,
        area: Rect,
        config: &'a Config,
        theme: &'a Theme,
        cursor: Cursor,
        is_scrollable: bool,
    ) -> Self {
        Self {
            _id,
            text_object,
            gutter: get_gutter(config, theme, area.clone()),
            area,
            config,
            theme,
            cursor,
            highlighter: Highlight::new(theme),
            scroll: Position::default(),
            is_scrollable,
        }
    }

    pub fn get_file_name(&self) -> String {
        self.text_object.borrow().file_name.clone()
    }

    fn handle_cursor_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        self.cursor
            .handle(action, &mut self.text_object.borrow_mut(), mode);

        if self.is_scrollable {
            self.maybe_scroll();
        } else {
            self.keep_cursor_in_viewport();
        }

        Ok(())
    }

    fn keep_cursor_in_viewport(&mut self) {
        match (self.cursor.col, self.cursor.row) {
            (x, _) if x as u16 >= self.area.width => {
                self.cursor.col = self.area.width.saturating_sub(1) as usize;
            }
            (_, y) if y as u16 >= self.area.height => {
                self.cursor.row = self.area.height.saturating_sub(1) as usize;
            }
            _ => (),
        }
    }

    fn handle_text_object_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        let col = self.cursor.col;
        let row = self.cursor.row;
        let mark = {
            let buffer = self.text_object.borrow_mut();
            let mark = buffer.marker.get_by_cursor(self.cursor.absolute_position);
            mark.unwrap()
        };

        self.text_object
            .borrow_mut()
            .handle_action(action, self.cursor.absolute_position)?;

        self.handle_cursor_action(action, mode)?;

        if let KeyAction::Simple(Action::DeletePreviousChar) = action {
            if let (0, 1..) = (col, row) {
                self.cursor.col = mark.size.saturating_sub(1);
                self.cursor.absolute_position = mark.start + mark.size.saturating_sub(1);
            }
        };

        Ok(())
    }

    fn apply_highlights(&mut self) -> Vec<Cell> {
        let mut cells = vec![];
        let text = self
            .text_object
            .borrow()
            .content_from(self.scroll.row, self.area.height as usize);
        let colors = self.highlighter.colors(&text);
        let mut i = 0;

        for c in text.chars() {
            let cell = match colors
                .iter()
                .find(|token| i >= token.start && i < token.end)
            {
                Some(token) => Cell {
                    c,
                    style: *token.style,
                },
                None => Cell {
                    c,
                    style: self.theme.appearance,
                },
            };

            cells.push(cell);
            i += c.len_utf8();
        }

        cells
    }
}

impl Renderable<'_> for FocusableBuffer<'_> {
    fn render(&mut self, frame: &mut crate::frame::Frame) -> anyhow::Result<()> {
        let gutter = match self.config.line_numbers {
            crate::config::LineNumbers::None => 0,
            _ => self.gutter.width(),
        };

        render_within_bounds(
            &self.apply_highlights(),
            frame,
            self.area.y,
            self.area.x,
            gutter,
            |col| {
                col > self.scroll.col as u16
                    && col - (self.scroll.col as u16) <= self.area.width - gutter
            },
        );

        self.gutter.render(
            frame,
            self.text_object.borrow().len(),
            self.area.height as usize,
            self.scroll.row,
        );

        Ok(())
    }

    fn resize(&mut self, new_area: Rect) -> anyhow::Result<()> {
        self.area = new_area;
        Ok(())
    }
}

impl Focusable<'_> for FocusableBuffer<'_> {
    fn render_cursor(&self, mode: &Mode) -> anyhow::Result<()> {
        let gutter_size = self.config.gutter_width;

        let col = {
            let mut col = 0;
            let marker = &self.text_object.borrow().marker;
            let Position { row, .. } = self.cursor.get_readable_position();

            if let Some(mark) = marker.get_by_line(row) {
                col = match mode {
                    Mode::Normal => self.cursor.col.min(mark.size.saturating_sub(2)),
                    _ => self.cursor.col.min(mark.size.saturating_sub(1)),
                }
            }

            col
        };

        let col = col.saturating_sub(self.scroll.col) + gutter_size;
        let row = self.cursor.row.saturating_sub(self.scroll.row);

        crossterm::queue!(stdout(), crossterm::cursor::MoveTo(col as u16, row as u16))?;

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
            KeyAction::Simple(Action::MoveToBottom) => {
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
            _ => tracing::debug!("[FocusableBuffer] unhandled action: {:?}", action),
        };

        Ok(())
    }
}

impl<'a> Scrollable<'a> for FocusableBuffer<'_> {
    fn maybe_scroll(&mut self) {
        // TODO: we are not handling when the user moves to a shorter line
        // in which the last character is not in the viewport.

        let gutter_width = match self.config.line_numbers {
            crate::config::LineNumbers::None => 0,
            _ => self.gutter.width(),
        };
        let Rect { width, height, .. } = &self.area;

        let col = self.cursor.col;

        match (col, self.cursor.row) {
            // should scroll down
            (_, y) if (y + 1).saturating_sub(self.scroll.row) >= *height as usize => {
                self.scroll.row = y + 1 - *height as usize;
            }
            // Should scroll up
            (_, y) if (y + 1).saturating_sub(self.scroll.row) == 0 => {
                self.scroll.row = self.scroll.row - (self.scroll.row - y);
            }
            // Should scroll right
            (x, _)
                if (x + gutter_width as usize).saturating_sub(self.scroll.col)
                    >= *width as usize =>
            {
                self.scroll.col = x + 1 + gutter_width as usize - *width as usize;
            }
            // Should scroll left
            (x, _) if (x + 1).saturating_sub(self.scroll.col) == 0 => {
                self.scroll.col = self.scroll.col - (self.scroll.col - x);
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::Frame;

    #[test]
    fn test_horizontal_scroll() {
        let config = Config::default();
        let theme = Theme::default();
        let text_object = Rc::new(RefCell::new(TextObject::from_string(1, "Hello, World!", 1)));
        let area = Rect::new(0, 0, 10, 2);
        let cursor = Cursor::default();

        let mut buffer = FocusableBuffer::new(1, text_object, area, &config, &theme, cursor, true);

        let mut frame = Frame::new(10, 2);
        buffer.render(&mut frame).unwrap();

        frame.cells.iter().for_each(|c| println!("{}", c.c));

        assert_eq!(frame.cells[6].c, 'H');
        assert_eq!(frame.cells[9].c, 'l');
        assert_eq!(buffer.cursor.col, 0);
        assert_eq!(buffer.scroll.col, 0);

        for _ in 0..6 {
            buffer
                .handle_cursor_action(&KeyAction::Simple(Action::MoveRight), &Mode::Normal)
                .unwrap();
        }

        let mut frame = Frame::new(10, 2);
        buffer.render(&mut frame).unwrap();

        frame.cells.iter().for_each(|c| println!("{}", c.c));

        assert_eq!(buffer.cursor.col, 6);
        assert_eq!(buffer.scroll.col, 3);
        assert_eq!(frame.cells[6].c, 'l');
    }
}
