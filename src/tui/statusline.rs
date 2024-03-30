use crate::{
    editor::Mode,
    frame::Frame,
    theme::Theme,
    tui::{rect::Rect, Renderable},
};

#[derive(Debug)]
pub struct Statusline<'a> {
    area: Rect,
    pub cursor: (u16, u16),
    pub file_name: &'a str,
    pub mode: Mode,
    pub theme: &'a Theme,
}

impl<'a> Statusline<'a> {
    pub fn new(area: Rect, theme: &'a Theme) -> Self {
        Self {
            area,
            theme,
            cursor: Default::default(),
            file_name: Default::default(),
            mode: Default::default(),
        }
    }

    pub fn update(&mut self, cursor: (u16, u16), file_name: &'a str, mode: Mode) {
        self.cursor = cursor;
        self.file_name = file_name;
        self.mode = mode;
    }
}

impl<'a> Renderable<'a> for Statusline<'_> {
    fn render(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        let mode = format!(" [{}] ", self.mode);
        let mode_gap = " ";
        let file_name = self.file_name;
        let cursor = format!("{}:{} ", self.cursor.1, self.cursor.0);
        let style = self.theme.statusline;
        let remaining_space = [mode.len(), mode_gap.len(), file_name.len(), cursor.len()]
            .iter()
            .fold(self.area.width as usize, |acc, len| acc - *len);
        let gap = " ".repeat(remaining_space);

        let mut col = self.area.x;
        frame.set_text(col, self.area.y, &mode, &style.mode);
        col += mode.len() as u16;
        frame.set_text(col, self.area.y, mode_gap, &style.appearance);
        col += mode_gap.len() as u16;
        frame.set_text(col, self.area.y, file_name, &style.file_name);
        col += file_name.len() as u16;
        frame.set_text(col, self.area.y, &gap, &style.appearance);
        col += remaining_space as u16;
        frame.set_text(col, self.area.y, &cursor, &style.cursor);

        Ok(())
    }

    fn resize(&mut self, new_area: Rect) -> anyhow::Result<()> {
        self.area = new_area;
        Ok(())
    }
}
