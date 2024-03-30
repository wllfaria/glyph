pub mod buffer;
pub mod layout;
pub mod rect;
pub mod statusline;

mod tui_view;

use crate::{config::KeyAction, editor::Mode, frame::Frame};

use self::rect::Rect;

// fn maybe_scroll(&mut self, cursor: &Cursor) {
//     let Rect { width, height, .. } = self.get_area();
//     let mut scroll = self.get_scroll().clone();
//     // all the instances of `y + 1` or `x + 1` are just normalizing the row/col to be 1 indexed
//     match (cursor.col, cursor.row) {
//         // should scroll down
//         (_, y) if (y + 1).saturating_sub(scroll.row) >= *height => {
//             scroll.row = y + 1 - height;
//         }
//         // Should scroll up
//         (_, y) if (y + 1).saturating_sub(scroll.row) == 0 => {
//             tracing::error!("mths {} {}", (y + 1).saturating_sub(scroll.row), y);
//             scroll.row -= scroll.row - y;
//         }
//         // Should scroll right
//         (x, _) if x.saturating_sub(scroll.col) >= *width => {
//             scroll.col = x + 1 - width;
//         }
//         // Should scroll left
//         (x, _) if (x + 1).saturating_sub(scroll.col) == 0 => {
//             scroll.col -= scroll.col - x;
//         }
//         _ => (),
//     }
//     self.set_scroll(scroll.clone());
// }
//

pub trait Renderable<'a> {
    fn render(&mut self, frame: &mut Frame) -> anyhow::Result<()>;
    fn resize(&mut self, new_area: Rect) -> anyhow::Result<()>;
}

pub trait Focusable<'a>: Renderable<'a> {
    fn render_cursor(&self, mode: &Mode) -> anyhow::Result<()>;
    fn handle_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()>;
}

pub trait Scrollable<'a>: Focusable<'a> {
    fn maybe_scroll(&mut self);
}
