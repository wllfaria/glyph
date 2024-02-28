use crate::cursor::Cursor;
use crate::pane::{Position, Rect};
use crate::viewport::{Cell, Viewport};

mod hover_popup;
mod tui_view;
pub use hover_popup::HoverPopup;
pub use tui_view::TuiView;

pub trait Scrollable: Renderable {
    fn maybe_scroll(
        &mut self,
        cursor: &Cursor,
        scroll: &mut Position,
        area: &Rect,
    ) -> anyhow::Result<()> {
        let height = area.height;
        let width = area.width;
        match (cursor.col, cursor.row) {
            // Should scroll down
            (_, y) if (y + 1).saturating_sub(scroll.row) >= height => {
                scroll.row = y + 1 - height;
            }
            // Should scroll up
            (_, y) if (y + 1).saturating_sub(scroll.row) == 0 => {
                scroll.row -= scroll.row - y;
            }
            // Should scroll right
            (x, _) if x.saturating_sub(scroll.col) >= width => {
                scroll.col = x + 1 - width;
            }
            // Should scroll left
            (x, _) if (x + 1).saturating_sub(scroll.col) == 0 => {
                scroll.col -= scroll.col - x;
            }
            _ => (),
        }
        Ok(())
    }
}

pub trait Renderable {
    fn render_diff(&mut self, last_view: &Viewport, view: &Viewport) -> anyhow::Result<()>;
    fn draw(&self, view: &mut Viewport, cells: &[Cell], scroll: &Position);
    fn resize(&mut self, new_area: Rect, offset: usize);
}
