use std::io::{stdout, Stdout};

use crossterm::{cursor, style, QueueableCommand};

use crate::pane::{Position, Rect};
use crate::theme::Theme;
use crate::tui::{Renderable, Scrollable};
use crate::viewport::{Cell, Viewport};

pub struct TuiView<'a> {
    stdout: Stdout,
    area: Rect,
    theme: &'a Theme,
    content_offset: usize,
}

impl<'a> TuiView<'a> {
    pub fn new(area: Rect, theme: &'a Theme, content_offset: usize) -> Self {
        Self {
            stdout: stdout(),
            area,
            theme,
            content_offset,
        }
    }
}

impl Scrollable for TuiView<'_> {}

impl Renderable for TuiView<'_> {
    fn render_diff(&mut self, last_view: &Viewport, view: &Viewport) -> anyhow::Result<()> {
        self.stdout.queue(crossterm::cursor::SavePosition)?;
        let changes = view.diff(last_view);

        for change in changes {
            let col = self.area.col + change.col;
            let row = self.area.row + change.row;

            self.stdout.queue(cursor::MoveTo(col as u16, row as u16))?;

            match change.cell.style.bg {
                Some(bg) => self.stdout.queue(style::SetBackgroundColor(bg))?,
                None => self
                    .stdout
                    .queue(style::SetBackgroundColor(self.theme.style.bg.unwrap()))?,
            };

            match change.cell.style.fg {
                Some(fg) => self.stdout.queue(style::SetForegroundColor(fg))?,
                None => self
                    .stdout
                    .queue(style::SetForegroundColor(self.theme.style.fg.unwrap()))?,
            };

            self.stdout.queue(style::Print(change.cell.c))?;
        }

        self.stdout.queue(crossterm::cursor::RestorePosition)?;

        Ok(())
    }

    fn draw(&self, view: &mut Viewport, cells: &[Cell], scroll: &Position) {
        let mut row = 0;
        let mut col = self.content_offset;

        for cell in cells {
            if col >= scroll.col && col - scroll.col < self.area.width {
                // we print a space when the char is a newline so the background gets printed
                match cell.c {
                    '\n' => view.set_cell(col - scroll.col, row, ' ', &cell.style),
                    _ => view.set_cell(col - scroll.col, row, cell.c, &cell.style),
                };
                col += 1;
            }

            if cell.c == '\n' {
                row += 1;
                col = self.content_offset;
            }
        }
    }

    fn resize(&mut self, new_area: Rect, offset: usize) {
        self.area = new_area;
        self.content_offset = offset;
    }
}
