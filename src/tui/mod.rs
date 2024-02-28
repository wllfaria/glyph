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
        // normalizing the row/col with line numbers
        let (col, row) = (cursor.col + 1, cursor.row + 1);

        match (col, row) {
            (_, y) if y.saturating_sub(scroll.row) >= height => {
                scroll.row = y - height;
            }
            (_, y) if y.saturating_sub(scroll.row) == 0 => {
                scroll.row -= scroll.row - y - 1;
            }
            (x, _) if x.saturating_sub(scroll.col) >= width => {
                scroll.col = x - width;
            }
            (x, _) if x.saturating_sub(scroll.col) == 0 => {
                scroll.col -= scroll.col - x - 1;
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
