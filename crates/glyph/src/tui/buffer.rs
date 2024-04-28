use crate::{
    cursor::Cursor,
    frame::{cell::Cell, Frame},
    highlight::Highlight,
    tui::{
        gutter::{get_gutter, GutterKind},
        position::Position,
        rect::Rect,
        Focusable, Renderable, Scrollable,
    },
};
use config::{Action, Config, KeyAction, LineNumbers, Mode};
use std::{cell::RefCell, io::stdout, marker::PhantomData, rc::Rc};
use text_object::TextObject;
use theme::Theme;

pub struct Regular;
pub struct WithCursor;

pub struct Buffer<'a, Kind = Regular> {
    id: usize,
    pub text_object: Rc<RefCell<TextObject>>,
    area: Rect,
    theme: &'a Theme,
    config: &'a Config,
    pub gutter: Option<GutterKind<'a>>,
    highlighter: Highlight<'a>,
    is_float: bool,
    kind: PhantomData<Kind>,
    pub cursor: Cursor,
    pub scroll: Position,
}

impl<'a> Buffer<'a, Regular> {
    pub fn new(
        id: usize,
        text_object: Rc<RefCell<TextObject>>,
        area: Rect,
        config: &'a Config,
        theme: &'a Theme,
        is_float: bool,
    ) -> Self {
        let gutter = if is_float {
            None
        } else {
            Some(get_gutter(config, theme, area.clone()))
        };
        Self {
            id,
            text_object,
            theme,
            gutter,
            area,
            config,
            highlighter: Highlight::new(theme),
            is_float,
            kind: PhantomData::<Regular>,
            cursor: Cursor::default(),
            scroll: Position::default(),
        }
    }

    pub fn with_cursor(self) -> Buffer<'a, WithCursor> {
        Buffer::<'a, WithCursor> {
            id: self.id,
            text_object: self.text_object,
            area: self.area,
            theme: self.theme,
            gutter: self.gutter,
            config: self.config,
            highlighter: self.highlighter,
            is_float: self.is_float,
            kind: PhantomData::<WithCursor>,
            cursor: Cursor::default(),
            scroll: Position::default(),
        }
    }

    fn apply_highlights(&mut self) -> Vec<Cell> {
        let mut cells = vec![];
        let text = self
            .text_object
            .borrow()
            .content_from(0, self.area.height as usize);
        let colors = self.highlighter.colors(&text);
        let mut i = 0;

        for c in text.chars() {
            let mut style = match colors
                .iter()
                .find(|token| i >= token.start && i < token.end)
            {
                Some(token) => *token.style,
                None => self.theme.appearance,
            };

            if self.is_float {
                style.bg = self.theme.float.bg;
            }

            let cell = Cell { c, style };

            cells.push(cell);
            i += c.len_utf8();
        }

        cells
    }
}

impl Renderable<'_> for Buffer<'_, Regular> {
    fn render(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        let gutter_width = match self.config.line_numbers {
            LineNumbers::None => 0,
            _ => match &self.gutter {
                Some(gutter) => gutter.width(),
                None => 0,
            },
        };
        render_within_bounds(
            &self.apply_highlights(),
            frame,
            &self.area,
            gutter_width,
            |col| col < self.area.width,
        );

        if let Some(gutter) = &self.gutter {
            gutter.render(
                frame,
                self.text_object.borrow().marker.len(),
                self.cursor.row,
                0,
            );
        }

        Ok(())
    }

    fn resize(&mut self, new_area: Rect) -> anyhow::Result<()> {
        self.area = new_area;
        Ok(())
    }
}

impl Buffer<'_, WithCursor> {
    pub fn get_file_name(&self) -> String {
        self.text_object.borrow().file_name.clone()
    }

    fn handle_cursor_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        self.cursor
            .handle(action, &mut self.text_object.borrow_mut(), mode);
        self.maybe_scroll();
        self.keep_cursor_in_viewport();

        Ok(())
    }

    fn keep_cursor_in_viewport(&mut self) {
        match (self.cursor.col, self.cursor.row) {
            (x, _) if x as u16 >= self.area.width => {
                self.cursor.col = self.area.width.saturating_sub(1) as usize;
            }
            (_, y) if y.saturating_sub(self.scroll.row) as u16 >= self.area.height => {
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

impl Renderable<'_> for Buffer<'_, WithCursor> {
    fn render(&mut self, frame: &mut crate::frame::Frame) -> anyhow::Result<()> {
        tracing::debug!("{:?}", self.config.line_numbers);
        let gutter = match self.config.line_numbers {
            LineNumbers::None => 0,
            _ => self.gutter.as_ref().map(|g| g.width()).unwrap_or(0),
        };

        render_within_bounds(&self.apply_highlights(), frame, &self.area, gutter, |col| {
            col > self.scroll.col as u16
                && col - (self.scroll.col as u16) <= self.area.width - gutter
        });

        if let Some(gutter) = &self.gutter {
            gutter.render(
                frame,
                self.text_object.borrow().marker.len(),
                self.cursor.row,
                self.scroll.row,
            )
        }

        Ok(())
    }

    fn resize(&mut self, new_area: Rect) -> anyhow::Result<()> {
        self.area = new_area;
        Ok(())
    }
}

impl Focusable<'_> for Buffer<'_, WithCursor> {
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

    #[tracing::instrument(skip(self))]
    fn handle_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        tracing::debug!("handling action");

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

impl Scrollable<'_> for Buffer<'_, WithCursor> {
    fn maybe_scroll(&mut self) {
        // TODO: we are not handling when the user moves to a shorter line
        // in which the last character is not in the viewport.

        let gutter_width = match self.config.line_numbers {
            LineNumbers::None => 0,
            _ => self.gutter.as_ref().map(|g| g.width()).unwrap_or(0),
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

fn render_within_bounds<F>(
    cells: &[Cell],
    frame: &mut Frame,
    area: &Rect,
    offset: u16,
    is_within_bounds: F,
) where
    F: Fn(u16) -> bool,
{
    let mut col = 0;
    let mut row = 0;
    let mut i = 1;

    for cell in cells {
        if is_within_bounds(i) {
            match cell.c {
                '\n' => frame.set_cell(col + area.x + offset, row + area.y, ' ', &cell.style),
                _ => frame.set_cell(col + area.x + offset, row + area.y, cell.c, &cell.style),
            }
            col += 1;
        }

        for i in col..area.width - offset {
            frame.set_cell(i + area.x + offset, row + area.y, ' ', &cell.style);
        }

        i += 1;

        if cell.c == '\n' {
            row += 1;
            col = 0;
            i = 1;
        }
    }

    for i in row + 1..area.height {
        for j in offset..area.width {
            frame.set_cell(area.x + j, i + area.y, ' ', &Default::default());
        }
    }
}
