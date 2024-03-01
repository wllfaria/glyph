use std::io::{stdout, Stdout};

use crossterm::style;
use crossterm::{cursor, QueueableCommand};

use crate::buffer::Buffer;
use crate::pane::Position;
use crate::theme::Theme;
use crate::viewport::Viewport;
use crate::{pane::Rect, viewport::Cell};

use super::Renderable;

pub struct HoverPopup<'a> {
    area: Rect,
    scroll: Position,
    offset: usize,
    stdout: Stdout,
    content: Buffer,
    theme: &'a Theme,
}

impl Renderable for HoverPopup<'_> {
    fn draw(&self, view: &mut Viewport, cells: &[Cell]) {
        let mut col = 0;
        let mut row = 0;
        for cell in cells {
            match cell.c {
                '\n' => view.set_cell(col, row, ' ', &cell.style),
                _ => view.set_cell(col, row, cell.c, &cell.style),
            }
            col += 1;
            if col == self.area.width {
                row += 1;
                col = 0;
            }
        }
    }

    fn resize(&mut self, _: Rect, _: usize) {
        // we don't resize popups.
    }

    fn get_area(&self) -> &Rect {
        &self.area
    }
    fn get_scroll(&self) -> &Position {
        &self.scroll
    }
    fn set_scroll(&mut self, scroll: Position) {
        self.scroll = scroll;
    }
    fn get_offset(&self) -> usize {
        self.offset
    }
    fn get_stdout(&mut self) -> &mut Stdout {
        &mut self.stdout
    }
    fn render_diff(
        &mut self,
        last_view: &Viewport,
        view: &Viewport,
        default_style: &crate::theme::Style,
    ) -> anyhow::Result<()> {
        let changes = view.diff(last_view);

        for c in 0..self.area.width + 2 {
            let col = c + self.area.col;
            self.stdout
                .queue(cursor::MoveTo(
                    col as u16,
                    self.area.height as u16 + self.area.row as u16,
                ))?
                .queue(style::SetBackgroundColor(default_style.bg.unwrap()))?
                .queue(style::Print(' '))?;
        }

        for r in 0..self.area.height + 1 {
            self.stdout
                .queue(cursor::MoveTo(
                    0 + self.area.col as u16,
                    r as u16 + self.area.row as u16,
                ))?
                .queue(style::SetBackgroundColor(default_style.bg.unwrap()))?
                .queue(style::Print(' '))?;
            self.stdout
                .queue(cursor::MoveTo(
                    self.area.width as u16 + self.area.col as u16 + 1,
                    r as u16 + self.area.row as u16,
                ))?
                .queue(style::SetBackgroundColor(default_style.bg.unwrap()))?
                .queue(style::Print(' '))?;
        }

        for change in changes {
            let col = change.col + self.area.col + 1;
            let row = change.row + self.area.row;

            match change.cell.style.bg {
                Some(bg) => self.stdout.queue(style::SetBackgroundColor(bg))?,
                None => self
                    .stdout
                    .queue(style::SetBackgroundColor(default_style.bg.unwrap()))?,
            };

            match change.cell.style.fg {
                Some(fg) => self.stdout.queue(style::SetForegroundColor(fg))?,
                None => self
                    .stdout
                    .queue(style::SetForegroundColor(default_style.fg.unwrap()))?,
            };

            self.stdout
                .queue(cursor::MoveTo(col as u16, row as u16))?
                .queue(style::Print(change.cell.c))?;
        }

        Ok(())
    }
}

impl<'a> HoverPopup<'a> {
    pub fn new(col: usize, row: usize, theme: &'a Theme, content: String) -> Self {
        let buffer = Buffer::from_string(0, &content, 0);
        let area = HoverPopup::calculate_area(&buffer, col, row);
        Self {
            theme,
            offset: 0,
            scroll: Position { row: 0, col: 0 },
            stdout: stdout(),
            content: buffer,
            area: area.clone(),
        }
    }

    fn calculate_area(buffer: &Buffer, col: usize, row: usize) -> Rect {
        let lines = buffer.marker.len();
        let height = lines.min(30);
        let mut width = 0;

        for line in buffer.lines() {
            width = width.max(line.len());
        }

        Rect {
            height,
            width,
            row: row + 1,
            col,
        }
    }

    pub fn render(&mut self) -> anyhow::Result<Viewport> {
        let cells = self.content_to_vec_cells();
        let mut view = Viewport::new(self.area.width, self.area.height);
        self.draw(&mut view, &cells);
        self.render_diff(&Viewport::new(0, 0), &view, &self.theme.float)?;
        Ok(view)
    }

    fn content_to_vec_cells(&self) -> Vec<Cell> {
        let style = self.theme.float;
        let mut cells = vec![Cell::default(); self.area.width * self.area.height];
        let mut col = 0;
        let mut row = 0;
        for c in self.content.to_string().chars() {
            let pos = row * self.area.width + col;
            match c {
                '\n' => {
                    cells[pos] = Cell { c, style };
                    row += 1;
                    col = 0;
                }
                _ => {
                    cells[pos] = Cell { c, style };
                    col += 1;
                }
            }
        }
        tracing::error!("{:?}", cells.iter().rev().nth(0));
        cells
    }
}
