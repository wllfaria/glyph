use std::{cell::RefCell, io::stdout, rc::Rc};

use crate::{
    buffer::TextObject,
    config::{Config, KeyAction},
    cursor::Cursor,
    editor::Mode,
    frame::cell::Cell,
    highlight::Highlight,
    pane::{
        gutter::{get_gutter, GutterKind},
        Position,
    },
    theme::Theme,
    tui::{rect::Rect, Focusable, Renderable, Scrollable},
};

pub struct ScrollableBuffer<'a> {
    cursor: Cursor,
    id: usize,
    text_object: Rc<RefCell<TextObject>>,
    area: Rect,
    config: &'a Config,
    theme: &'a Theme,
    gutter: GutterKind<'a>,
    highlighter: Highlight<'a>,
    scroll: Position,
}

impl<'a> ScrollableBuffer<'a> {
    pub fn new(
        id: usize,
        text_object: Rc<RefCell<TextObject>>,
        area: Rect,
        config: &'a Config,
        theme: &'a Theme,
        cursor: Cursor,
    ) -> Self {
        Self {
            id,
            text_object,
            gutter: get_gutter(config, theme, area.clone()),
            area,
            config,
            theme,
            highlighter: Highlight::new(theme),
            cursor,
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

impl Renderable<'_> for ScrollableBuffer<'_> {
    fn render(&mut self, frame: &mut crate::frame::Frame) -> anyhow::Result<()> {
        let cells = self.apply_highlights();
        let mut row = self.area.y;
        let mut col = self.gutter.width();

        for cell in cells {
            if col >= self.scroll.col as u16 && col - (self.scroll.col as u16) < self.area.width {
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

impl Focusable<'_> for ScrollableBuffer<'_> {
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

        crossterm::queue!(
            stdout(),
            crossterm::cursor::MoveTo(col as u16 + gutter_size as u16, self.cursor.row as u16,)
        )?;

        Ok(())
    }

    fn handle_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Scrollable<'_> for ScrollableBuffer<'_> {
    fn maybe_scroll(&mut self) {
        let Rect { width, height, .. } = &self.area;
        // // all the instances of `y + 1` or `x + 1` are just normalizing the row/col to be 1 indexed
        match (self.cursor.col, self.cursor.row) {
            //     // should scroll down
            (_, y) if (y + 1).saturating_sub(self.scroll.row) >= *height as usize => {
                self.scroll.row = y + 1 - *height as usize;
            }
            // Should scroll up
            (_, y) if (y + 1).saturating_sub(self.scroll.row) == 0 => {
                tracing::error!("mths {} {}", (y + 1).saturating_sub(self.scroll.row), y);
                self.scroll.row = self.scroll.row - (self.scroll.row - y);
            }
            // Should scroll right
            (x, _) if x.saturating_sub(self.scroll.col) >= *width as usize => {
                self.scroll.col = x + 1 - *width as usize;
            }
            // Should scroll left
            (x, _) if (x + 1).saturating_sub(self.scroll.col) == 0 => {
                self.scroll.col = self.scroll.col - (self.scroll.col - x);
            }
            _ => (),
        }
    }
}
