mod focusable_buffer;

pub use focusable_buffer::FocusableBuffer;

use std::{cell::RefCell, rc::Rc};

use crate::{
    buffer::TextObject,
    config::Config,
    cursor::Cursor,
    frame::{cell::Cell, Frame},
    highlight::Highlight,
    theme::Theme,
    tui::{gutter::GutterKind, rect::Rect, Renderable},
};

pub struct Buffer<'a> {
    _id: usize,
    text_object: Rc<RefCell<TextObject>>,
    area: Rect,
    theme: &'a Theme,
    gutter: GutterKind<'a>,
    highlighter: Highlight<'a>,
}

impl<'a> Buffer<'a> {
    pub fn focusable(
        id: usize,
        text_object: Rc<RefCell<TextObject>>,
        area: Rect,
        config: &'a Config,
        theme: &'a Theme,
        is_scrollable: bool,
    ) -> FocusableBuffer<'a> {
        FocusableBuffer::new(
            id,
            text_object,
            area,
            config,
            theme,
            Cursor::default(),
            is_scrollable,
        )
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
    fn render(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        render_within_bounds(&self.apply_highlights(), frame, &self.area, 0, |col| {
            col < self.area.width
        });

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

    tracing::trace!("filling remaining cells from: {} to {}", row, area.height);
    for i in row + 1..area.height {
        for j in offset..area.width {
            frame.set_cell(area.x + j, i + area.y, ' ', &Default::default());
        }
    }
}
