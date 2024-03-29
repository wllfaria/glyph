use crate::{
    editor::Mode,
    theme::Theme,
    tui::{
        rect::Rect,
        statusline::{Statusline, StatuslineContext},
        Renderable,
    },
    window::Window,
};

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

pub struct LayoutUpdate {
    pub cursor_position: (u16, u16),
    pub current_file_name: String,
    pub mode: Mode,
}

pub struct Layout<'a> {
    cursor_position: (u16, u16),
    current_file_name: String,
    mode: Mode,

    theme: &'a Theme,
    // left_panel: Option<Widget>,
    // right_panel: Option<Widget>,
    // top_panel: Option<Widget>,
    // bottom_panel: Option<Widget>,
    // middle_panel: Option<Widget>,
    // bufferline: Option<Widget>,
    statusline: Statusline,
    // commandline: Widget,
    editor_window: &'a Window<'a>,
    focused: Sections,
}

impl<'a> Layout<'a> {
    pub fn new(
        size: Rect,
        theme: &'a Theme,
        layout_update: LayoutUpdate,
        window: &'a Window,
    ) -> Self {
        Self {
            cursor_position: layout_update.cursor_position,
            current_file_name: layout_update.current_file_name,
            mode: layout_update.mode,

            theme,
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
            editor_window: window,
            focused: Sections::Statusline,
        }
    }
}
