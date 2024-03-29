use std::io::stdout;

use crossterm::style::Stylize;

use crate::{
    editor::Mode,
    theme::{StatuslineStyle, Style},
};

use super::{rect::Rect, Renderable};

pub enum Sections {
    LeftPanel,
    RightPanel,
    TopPanel,
    BottomPanel,
    MiddlePanel,
    Bufferline,
    Statusline,
    Commandline,
    EditorWindow,
}

pub struct Layout {
    // left_panel: Option<Widget>,
    // right_panel: Option<Widget>,
    // top_panel: Option<Widget>,
    // bottom_panel: Option<Widget>,
    // middle_panel: Option<Widget>,
    // bufferline: Option<Widget>,
    statusline: Statusline,
    // commandline: Widget,
    // editor_window: Widget,
    focused: Sections,
}

pub struct Statusline {
    area: Rect,
}

impl Statusline {
    pub fn new(area: Rect) -> Self {
        Self { area }
    }
}

pub struct StatuslineContext {
    cursor: (u16, u16),
    file_name: String,
    mode: Mode,
    statusline_style: StatuslineStyle,
}

pub trait Themed {
    fn themed(self, style: Style) -> crossterm::style::StyledContent<String>;
}

impl Themed for String {
    fn themed(self, style: Style) -> crossterm::style::StyledContent<String> {
        let mut styled = self;
        if let Some(fg) = style.fg {
            styled = styled.with(fg);
        }

        if let Some(bg) = style.bg {
            styled = styled.on(bg);
        }

        if let Some(italic) = style.italic {
            styled = styled.italic();
        }

        if let Some(bold) = style.bold {
            styled = styled.bold();
        }
    }
}

impl Renderable for Statusline {
    type RenderContext = StatuslineContext;

    fn render(&self, context: &Self::RenderContext) -> anyhow::Result<()> {
        crossterm::queue!(
            stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
            crossterm::cursor::MoveTo(self.area.x, self.area.y),
        )?;

        let mode = format!("[{}]", context.mode).themed(context.statusline_style.mode);
        let cursor = format!("{}:{}", context.cursor.1, context.cursor.0)
            .themed(context.statusline_style.cursor);

        crossterm::execute!(
            stdout(),
            crossterm::style::Print(mode),
            crossterm::style::Print(" "),
            crossterm::style::Print(&context.file_name),
            crossterm::style::Print(" "),
            crossterm::style::Print(cursor),
        )?;

        Ok(())
    }
}

impl Layout {
    pub fn new(size: Rect) -> Self {
        Self {
            // left_panel: None,
            // right_panel: None,
            // top_panel: None,
            // bottom_panel: None,
            // middle_panel: None,
            // bufferline: None,
            statusline: Statusline::new(Rect::new(size.x, size.bottom() - 2, size.width, 1)),
            // commandline: Widget {
            //     state: WidgetState::Visible,
            //     area: Rect::new(size.x, size.bottom().saturating_sub(1), size.width, 1),
            // },
            // editor_window: Widget {
            //     state: WidgetState::Visible,
            //     area: Rect::new(size.x, size.y, size.width, size.height.saturating_sub(2)),
            // },
            focused: Sections::Statusline,
        }
    }

    pub fn render(&self) -> anyhow::Result<()> {
        self.statusline.render(&StatuslineContext {
            cursor: (0, 0),
            file_name: "test.rs".to_string(),
            mode: Mode::Normal,
            statusline_style: StatuslineStyle::default(),
        })?;

        Ok(())
    }
}
