use crate::{
    config::KeyAction,
    cursor::Cursor,
    editor::Mode,
    tui::{
        buffer::focusable_buffer::FocusableBuffer, rect::Rect, Focusable, Renderable, Scrollable,
    },
};

pub struct ScrollableBuffer<'a> {
    buffer: FocusableBuffer<'a>,
    cursor: Cursor,
}

impl<'a> ScrollableBuffer<'a> {
    pub fn new(buffer: FocusableBuffer<'a>, cursor: Cursor) -> Self {
        Self { buffer, cursor }
    }
}

impl Renderable<'_> for ScrollableBuffer<'_> {
    fn render(&mut self, frame: &mut crate::frame::Frame) -> anyhow::Result<()> {
        self.buffer.render(frame)?;
        Ok(())
    }

    fn resize(&mut self, new_area: Rect) -> anyhow::Result<()> {
        self.buffer.resize(new_area)?;

        Ok(())
    }
}

impl Focusable<'_> for ScrollableBuffer<'_> {
    fn render_cursor(&self, mode: &Mode) -> anyhow::Result<()> {
        self.buffer.render_cursor(mode)?;
        Ok(())
    }

    fn handle_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Scrollable<'_> for ScrollableBuffer<'_> {
    fn maybe_scroll(&mut self) {}
}
