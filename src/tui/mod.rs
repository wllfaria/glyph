pub mod buffer;
pub mod commandline;
pub mod gutter;
pub mod position;
pub mod rect;
pub mod statusline;

use crate::{
    config::{Config, KeyAction},
    cursor::Cursor,
    editor::Mode,
    frame::Frame,
    text_object::TextObject,
    theme::Theme,
    tui::{buffer::Buffer, rect::Rect},
};

use std::{cell::RefCell, rc::Rc};

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

fn calculate_popup_width(text: &str, gutter: u16, area: &Rect, cursor_col: usize) -> (u16, u16) {
    let mut text_max_width = text.lines().map(|l| l.len()).max().unwrap_or(0) as u16;
    let x = if text_max_width > area.width - gutter {
        0
    } else if cursor_col as u16 + text_max_width > area.width - gutter {
        area.width - text_max_width
    } else {
        cursor_col as u16 + gutter
    };
    text_max_width = u16::min(text_max_width, area.width);
    (x, text_max_width)
}

fn calculate_popup_height(text: &str, area: &Rect, cursor_row: usize) -> (u16, u16) {
    let text_max_height = text.lines().count() as u16;
    let y = if text_max_height > area.height {
        0
    } else if cursor_row as u16 + text_max_height > area.height {
        area.height - text_max_height
    } else {
        cursor_row as u16
    };
    let text_max_height = u16::min(text_max_height, area.height);
    (y, text_max_height)
}

pub fn create_popup<'a>(
    area: &Rect,
    gutter: u16,
    text: String,
    cursor: &Cursor,
    config: &'a Config,
    theme: &'a Theme,
) -> Buffer<'a> {
    let mut padded_text = text
        .lines()
        .map(|l| format!(" {} ", l))
        .collect::<Vec<String>>();
    padded_text.insert(0, " ".to_string());
    padded_text.push(" ".to_string());
    let text = padded_text.join("\n");
    let (x, width) = calculate_popup_width(&text, gutter, area, cursor.col);
    let (y, height) = calculate_popup_height(&text, area, cursor.row);

    let text_object = TextObject::from_string(1, &text, 10);

    Buffer::new(
        1,
        Rc::new(RefCell::new(text_object)),
        Rect::new(x, y, width, height),
        config,
        theme,
        true,
    )
}
