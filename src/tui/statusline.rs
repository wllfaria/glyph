use std::io::stdout;

use crate::{
    editor::Mode,
    theme::StatuslineTheming,
    tui::{rect::Rect, themed::Themed, Renderable},
};

pub struct Statusline {
    area: Rect,
}

impl Statusline {
    pub fn new(area: Rect) -> Self {
        Self { area }
    }
}

pub struct StatuslineContext {
    pub cursor: (u16, u16),
    pub file_name: String,
    pub mode: Mode,
    pub statusline_style: StatuslineTheming,
}

impl Renderable for Statusline {
    type RenderContext = StatuslineContext;

    fn render(&self, context: &Self::RenderContext) -> anyhow::Result<()> {
        let mode = format!("[{}]", context.mode);
        let mode_gap = " ".repeat(2);
        let file_name = context.file_name.clone();
        let cursor = format!("{}:{} ", context.cursor.1, context.cursor.0);

        let remaining_space =
            self.area.width as usize - mode.len() - mode_gap.len() - file_name.len() - cursor.len();

        let mode = mode.themed(context.statusline_style.mode);
        let mode_gap = mode_gap.themed(context.statusline_style.appearance);
        let file_name = file_name.themed(context.statusline_style.file_name);
        let cursor = cursor.themed(context.statusline_style.cursor);

        let padding = " "
            .repeat(remaining_space)
            .themed(context.statusline_style.appearance);

        crossterm::queue!(
            stdout(),
            crossterm::cursor::MoveTo(self.area.x, self.area.y),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
            crossterm::style::PrintStyledContent(mode),
            crossterm::style::PrintStyledContent(mode_gap),
            crossterm::style::PrintStyledContent(file_name),
            crossterm::style::PrintStyledContent(padding),
            crossterm::style::PrintStyledContent(cursor),
        )?;

        Ok(())
    }
}
