use std::{cell::RefCell, rc::Rc};

use crate::{
    buffer::TextObject,
    config::Config,
    cursor::Cursor,
    highlight::Highlight,
    pane::gutter::{get_gutter, GutterKind},
    theme::Theme,
    viewport::Cell,
};

use super::{rect::Rect, Focusable, Renderable, Scrollable};

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
        FocusableBuffer {
            buffer: Self::new(id, text_object, area, config, theme),
            cursor: Cursor::default(),
        }
    }

    pub fn scrollable(
        id: usize,
        text_object: Rc<RefCell<TextObject>>,
        area: Rect,
        config: &'a Config,
        theme: &'a Theme,
    ) -> ScrollableBuffer<'a> {
        ScrollableBuffer {
            buffer: Self::focusable(id, text_object, area, config, theme),
            cursor: Cursor::default(),
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

pub struct BufferRenderContext {
    scroll: usize,
}

impl Renderable<'_> for Buffer<'_> {
    fn render(&mut self, frame: &mut crate::viewport::Frame) -> anyhow::Result<()> {
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
}

pub struct FocusableBuffer<'a> {
    buffer: Buffer<'a>,
    cursor: Cursor,
}

impl Renderable<'_> for FocusableBuffer<'_> {
    fn render(&mut self, frame: &mut crate::viewport::Frame) -> anyhow::Result<()> {
        self.buffer.render(frame)?;
        Ok(())
    }
}

impl Focusable<'_> for FocusableBuffer<'_> {
    fn render_cursor(&self) -> anyhow::Result<()> {
        Ok(())
    }
    fn unfocus(&mut self) {}
    fn focus(&mut self) {}
}

pub struct ScrollableBuffer<'a> {
    buffer: FocusableBuffer<'a>,
    cursor: Cursor,
}

impl Renderable<'_> for ScrollableBuffer<'_> {
    fn render(&mut self, frame: &mut crate::viewport::Frame) -> anyhow::Result<()> {
        self.buffer.render(frame)?;
        Ok(())
    }
}

impl Focusable<'_> for ScrollableBuffer<'_> {
    fn render_cursor(&self) -> anyhow::Result<()> {
        self.buffer.render_cursor()?;
        Ok(())
    }

    fn unfocus(&mut self) {
        self.buffer.unfocus();
    }
    fn focus(&mut self) {
        self.buffer.focus();
    }
}

impl Scrollable<'_> for ScrollableBuffer<'_> {
    fn maybe_scroll(&mut self) {}
}
