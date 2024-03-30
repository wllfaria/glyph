use crate::{
    editor::Mode,
    theme::StatuslineTheming,
    tui::{rect::Rect, Renderable},
    viewport::Frame,
};

pub struct Statusline {
    area: Rect,
}

impl Statusline {
    pub fn new(area: Rect) -> Self {
        Self { area }
    }
}

pub struct StatuslineContext<'a> {
    pub cursor: (u16, u16),
    pub file_name: &'a str,
    pub mode: &'a Mode,
    pub statusline_style: &'a StatuslineTheming,
}

impl<'a> Renderable<'a> for Statusline {
    type RenderContext = StatuslineContext<'a>;

    fn render(&self, frame: &mut Frame, context: &Self::RenderContext) -> anyhow::Result<()> {
        let mode = format!("[{}]", context.mode);
        let mode_gap = " ".repeat(2);
        let file_name = context.file_name.clone();
        let cursor = format!("{}:{} ", context.cursor.1, context.cursor.0);

        let remaining_space = [mode.len(), mode_gap.len(), file_name.len(), cursor.len()]
            .iter()
            .fold(self.area.width as usize, |acc, len| acc - *len);

        let mut col = self.area.x;

        frame.set_text(col, self.area.y, &mode, &context.statusline_style.mode);
        col += mode.len() as u16;
        frame.set_text(
            col,
            self.area.y,
            &mode_gap,
            &context.statusline_style.appearance,
        );
        col += mode_gap.len() as u16;
        frame.set_text(
            col,
            self.area.y,
            &file_name,
            &context.statusline_style.file_name,
        );
        col += file_name.len() as u16;
        frame.set_text(
            col,
            self.area.y,
            &" ".repeat(remaining_space),
            &context.statusline_style.appearance,
        );
        col += remaining_space as u16;

        frame.set_text(col, self.area.y, &cursor, &context.statusline_style.cursor);

        Ok(())
    }
}
