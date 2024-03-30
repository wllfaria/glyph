pub mod buffer;
pub mod gutter;
pub mod position;
pub mod rect;
pub mod statusline;

mod tui_view;

use crate::{config::KeyAction, editor::Mode, frame::Frame};

use self::rect::Rect;

// fn maybe_scroll(&mut self, cursor: &Cursor) {
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
