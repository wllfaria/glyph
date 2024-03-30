use crate::viewport::Frame;

pub mod buffer;
pub mod layout;
pub mod rect;
pub mod statusline;
pub mod themed;

mod tui_view;

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
// fn draw_cursor(&mut self, mode: &Mode, buffer: &Buffer, cursor: &Cursor) -> anyhow::Result<()> {
//     let offset = self.get_offset();
//     let scroll = self.get_scroll().clone();
//
//     let stdout = self.get_stdout();
//
//     let col = {
//         let mut col = 0;
//         if let Some(mark) = buffer.marker.get_by_line(cursor.row + 1) {
//             col = match mode {
//                 Mode::Normal => cursor.col.min(mark.size.saturating_sub(2)),
//                 _ => cursor.col.min(mark.size.saturating_sub(1)),
//             };
//         }
//         col
//     };
//     stdout.queue(crossterm::cursor::MoveTo(
//         col.saturating_sub(scroll.col) as u16 + offset as u16,
//         cursor.row.saturating_sub(scroll.row) as u16,
//     ))?;
//
//     Ok(())
// }
//

pub trait Renderable<'a> {
    // fn render_diff(
    //     &mut self,
    //     last_view: &Viewport,
    //     view: &Viewport,
    //     default_style: &Style,
    // ) -> anyhow::Result<()>;
    fn render(&mut self, frame: &mut Frame) -> anyhow::Result<()>;
}

pub trait Focusable<'a>: Renderable<'a> {
    fn focus(&mut self);
    fn unfocus(&mut self);
    fn render_cursor(&self) -> anyhow::Result<()>;
}

pub trait Scrollable<'a>: Focusable<'a> {
    fn maybe_scroll(&mut self);
}
