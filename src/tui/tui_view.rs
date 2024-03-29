// use std::io::{stdout, Stdout};
//
// use crossterm::{cursor, style, QueueableCommand};
//
// use crate::pane::{Position, Rect};
// use crate::theme::Style;
// use crate::tui::{Renderable, Scrollable};
// use crate::viewport::{Cell, Viewport};
//
// pub struct TuiView {
//     stdout: Stdout,
//     area: Rect,
//     content_offset: usize,
//     scroll: Position,
// }
//
// impl TuiView {
//     pub fn new(area: Rect, content_offset: usize) -> Self {
//         Self {
//             stdout: stdout(),
//             area,
//             content_offset,
//             scroll: Position::default(),
//         }
//     }
// }
//
// impl Scrollable for TuiView {}
//
// impl Renderable for TuiView {
//     fn render_diff(
//         &mut self,
//         last_view: &Viewport,
//         view: &Viewport,
//         default_style: &Style,
//     ) -> anyhow::Result<()> {
//         let changes = view.diff(last_view);
//
//         for change in changes {
//             let col = self.area.col + change.col;
//             let row = self.area.row + change.row;
//
//             self.stdout.queue(cursor::MoveTo(col as u16, row as u16))?;
//
//             match change.cell.style.bg {
//                 Some(bg) => self.stdout.queue(style::SetBackgroundColor(bg))?,
//                 None => self
//                     .stdout
//                     .queue(style::SetBackgroundColor(default_style.bg.unwrap()))?,
//             };
//
//             match change.cell.style.fg {
//                 Some(fg) => self.stdout.queue(style::SetForegroundColor(fg))?,
//                 None => self
//                     .stdout
//                     .queue(style::SetForegroundColor(default_style.fg.unwrap()))?,
//             };
//
//             self.stdout.queue(style::Print(change.cell.c))?;
//         }
//
//         Ok(())
//     }
//
//     fn draw(&self, view: &mut Viewport, cells: &[Cell]) {
//         let mut row = 0;
//         let mut col = self.content_offset;
//         for cell in cells {
//             if col >= self.scroll.col && col - self.scroll.col < self.area.width {
//                 // we print a space when the char is a newline so the background gets printed
//                 match cell.c {
//                     '\n' => view.set_cell(col - self.scroll.col, row, ' ', &cell.style),
//                     _ => view.set_cell(col - self.scroll.col, row, cell.c, &cell.style),
//                 };
//                 col += 1;
//             }
//
//             if cell.c == '\n' {
//                 row += 1;
//                 col = self.content_offset;
//             }
//         }
//     }
//
//     fn resize(&mut self, new_area: Rect, offset: usize) {
//         self.area = new_area;
//         self.content_offset = offset;
//     }
//
//     fn get_area(&self) -> &Rect {
//         &self.area
//     }
//
//     fn get_scroll(&self) -> &Position {
//         &self.scroll
//     }
//
//     fn set_scroll(&mut self, scroll: Position) {
//         self.scroll = scroll;
//     }
//
//     fn get_offset(&self) -> usize {
//         self.content_offset
//     }
//
//     fn get_stdout(&mut self) -> &mut Stdout {
//         &mut self.stdout
//     }
// }
