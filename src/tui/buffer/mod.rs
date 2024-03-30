mod focusable_buffer;
mod scrollable_buffer;

pub use focusable_buffer::FocusableBuffer;
pub use scrollable_buffer::ScrollableBuffer;

use std::{cell::RefCell, rc::Rc};

use crate::{
    buffer::TextObject,
    config::Config,
    cursor::Cursor,
    frame::cell::Cell,
    highlight::Highlight,
    pane::gutter::{get_gutter, GutterKind},
    theme::Theme,
    tui::{rect::Rect, Renderable},
};

pub struct Buffer<'a> {
    id: usize,
    text_object: Rc<RefCell<TextObject>>,
    area: Rect,
    config: &'a Config,
    theme: &'a Theme,
    gutter: GutterKind<'a>,
    highlighter: Highlight<'a>,
}

impl<'a> Buffer<'a> {
    pub fn new(
        id: usize,
        text_object: Rc<RefCell<TextObject>>,
        area: Rect,
        config: &'a Config,
        theme: &'a Theme,
    ) -> Self {
        Self {
            id,
            text_object,
            gutter: get_gutter(config, theme, area.clone()),
            area,
            config,
            theme,
            highlighter: Highlight::new(theme),
        }
    }

    pub fn focusable(
        id: usize,
        text_object: Rc<RefCell<TextObject>>,
        area: Rect,
        config: &'a Config,
        theme: &'a Theme,
    ) -> FocusableBuffer<'a> {
        let buffer = Self::new(id, text_object, area, config, theme);
        FocusableBuffer::new(buffer, Cursor::default())
    }

    pub fn scrollable(
        id: usize,
        text_object: Rc<RefCell<TextObject>>,
        area: Rect,
        config: &'a Config,
        theme: &'a Theme,
    ) -> ScrollableBuffer<'a> {
        let buffer = Self::focusable(id, text_object, area, config, theme);
        ScrollableBuffer::new(buffer, Cursor::default())
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

impl Renderable<'_> for Buffer<'_> {
    fn render(&mut self, frame: &mut crate::frame::Frame) -> anyhow::Result<()> {
        let cells = self.apply_highlights();
        let mut row = self.area.y;
        let mut col = self.gutter.width();

        for cell in cells {
            if col < self.area.width {
                match cell.c {
                    '\n' => frame.set_cell(col, row, ' ', &cell.style),
                    _ => frame.set_cell(col, row, cell.c, &cell.style),
                }
                col += 1;
            }

            if cell.c == '\n' {
                row += 1;
                col = self.gutter.width();
            }
        }

        self.gutter.render(
            frame,
            self.text_object.borrow().len(),
            self.area.height as usize,
            0,
        );

        Ok(())
    }

    fn resize(&mut self, new_area: Rect) -> anyhow::Result<()> {
        self.area = new_area;

        Ok(())
    }
}
