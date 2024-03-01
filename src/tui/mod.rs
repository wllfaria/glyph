use std::io::Stdout;

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::editor::Mode;
use crate::pane::{Position, Rect};
use crate::theme::Style;
use crate::viewport::{Cell, Viewport};

mod hover_popup;
mod tui_view;
use crossterm::QueueableCommand;
pub use hover_popup::HoverPopup;
pub use tui_view::TuiView;

pub trait Scrollable: Renderable {
    fn maybe_scroll(
        &self,
        cursor: &Cursor,
        scroll: &Position,
        area: &Rect,
    ) -> anyhow::Result<Position> {
        let height = area.height;
        let width = area.width;
        let mut scroll = scroll.clone();
        // all the instances of `y + 1` or `x + 1` are just normalizing the row/col to be 1 indexed
        match (cursor.col, cursor.row) {
            // if the normalized_row
            (_, y) if (y + 1).saturating_sub(scroll.row) >= height => {
                scroll.row = y + 1 - height;
            }
            // Should scroll up
            (_, y) if (y + 1).saturating_sub(scroll.row) == 0 => {
                scroll.row -= y;
            }
            // Should scroll right
            (x, _) if x.saturating_sub(scroll.col) >= width => {
                scroll.col = x + 1 - width;
            }
            // Should scroll left
            (x, _) if (x + 1).saturating_sub(scroll.col) == 0 => {
                scroll.col -= x;
            }
            _ => (),
        }
        Ok(scroll)
    }

    fn draw_cursor(&mut self, mode: &Mode, buffer: &Buffer, cursor: &Cursor) -> anyhow::Result<()> {
        let offset = self.get_offset();
        let area = self.get_area();
        let scroll = self.get_scroll();

        let scroll = self.maybe_scroll(cursor, scroll, area)?;

        let stdout = self.get_stdout();

        let col = {
            let mut col = 0;
            if let Some(mark) = buffer.marker.get_by_line(cursor.row + 1) {
                col = match mode {
                    Mode::Normal => cursor.col.min(mark.size.saturating_sub(2)),
                    _ => cursor.col.min(mark.size.saturating_sub(1)),
                };
            }
            col
        };
        stdout.queue(crossterm::cursor::MoveTo(
            col.saturating_sub(scroll.col) as u16 + offset as u16,
            cursor.row.saturating_sub(scroll.row) as u16,
        ))?;

        self.set_scroll(scroll);
        Ok(())
    }

    fn hide_cursor(&mut self) -> anyhow::Result<()> {
        let stdout = self.get_stdout();
        stdout.queue(crossterm::cursor::Hide)?;
        Ok(())
    }

    fn show_cursor(&mut self) -> anyhow::Result<()> {
        let stdout = self.get_stdout();
        stdout.queue(crossterm::cursor::Show)?;
        Ok(())
    }
}

pub trait Renderable {
    fn render_diff(
        &mut self,
        last_view: &Viewport,
        view: &Viewport,
        default_style: &Style,
    ) -> anyhow::Result<()>;
    fn draw(&self, view: &mut Viewport, cells: &[Cell], scroll: &Position);
    fn resize(&mut self, new_area: Rect, offset: usize);
    fn get_area(&self) -> &Rect;
    fn get_scroll(&self) -> &Position;
    fn set_scroll(&mut self, scroll: Position);
    fn get_offset(&self) -> usize;
    fn get_stdout(&mut self) -> &mut Stdout;
}
