mod focusable_buffer;

pub use focusable_buffer::FocusableBuffer;

use std::{cell::RefCell, rc::Rc};

use crate::{
    buffer::TextObject,
    config::{Config, LineNumbers},
    cursor::Cursor,
    frame::{cell::Cell, Frame},
    highlight::Highlight,
    theme::Theme,
    tui::{gutter::GutterKind, rect::Rect, Renderable},
};

use super::gutter::get_gutter;

pub struct Buffer<'a> {
    id: usize,
    text_object: Rc<RefCell<TextObject>>,
    area: Rect,
    theme: &'a Theme,
    config: &'a Config,
    gutter: Option<GutterKind<'a>>,
    highlighter: Highlight<'a>,
    is_float: bool,
}

impl<'a> Buffer<'a> {
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
        }
    }

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

impl Renderable<'_> for Buffer<'_> {
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
                self.text_object.borrow().len(),
                self.area.height as usize,
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
