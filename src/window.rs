use std::collections::HashMap;

use crate::config::KeyAction;
use crate::editor::Mode;
use crate::lsp::IncomingMessage;
use crate::pane::{Pane, Rect};

pub struct Window<'a> {
    pub id: usize,
    pub panes: HashMap<usize, Pane<'a>>,
    pub active_pane: usize,
}

impl<'a> Window<'a> {
    pub fn new(id: usize, pane: Pane<'a>) -> Self {
        let mut panes = HashMap::new();
        let pane_id = pane.id;
        panes.insert(pane.id, pane);

        Self {
            id,
            active_pane: pane_id,
            panes,
        }
    }

    pub fn resize(&mut self, new_size: Rect, mode: &Mode) -> anyhow::Result<()> {
        for pane in self.panes.values_mut() {
            pane.resize(new_size.clone(), mode)?;
        }
        Ok(())
    }

    pub fn handle_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        let active_pane = self.panes.get_mut(&self.active_pane).unwrap();
        if let KeyAction::Simple(_) = action {
            active_pane.handle_action(action, mode)?;
        }
        Ok(())
    }

    pub fn handle_lsp_message(
        &mut self,
        message: (IncomingMessage, Option<String>),
    ) -> anyhow::Result<()> {
        let active_pane = self.panes.get_mut(&self.active_pane).unwrap();
        active_pane.handle_lsp_message(message)?;
        Ok(())
    }

    pub fn initialize(&mut self, mode: &Mode) -> anyhow::Result<()> {
        self.render_panes(mode)?;
        Ok(())
    }

    pub fn get_active_pane(&self) -> &Pane {
        self.panes.get(&self.active_pane).unwrap()
    }

    fn render_panes(&mut self, mode: &Mode) -> anyhow::Result<()> {
        for pane in self.panes.values_mut() {
            pane.initialize(mode)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::config::{Config, EditorBackground, Keys, LineNumbers};
    use crate::theme::Theme;

    fn get_config() -> Config {
        Config {
            gutter_width: 6,
            theme: "".into(),
            keys: Keys::default(),
            log_file: None,
            background: EditorBackground::Dark,
            line_numbers: LineNumbers::Absolute,
            empty_line_char: '~',
            show_diagnostics: true,
            mouse_scroll_lines: None,
        }
    }

    #[test]
    fn test_resizing() {
        let buffer = Buffer::new(1, None).unwrap();
        let theme = Theme::default();
        let config = get_config();
        let pane = Pane::new(1, Rc::new(RefCell::new(buffer)), &theme, &config);
        let mut window = Window::new(1, pane);

        assert_eq!(
            window.get_active_pane().size,
            Rect {
                row: 0,
                col: 0,
                height: 1,
                width: 1
            }
        );

        window.resize((0, 0).into(), &Mode::Normal).unwrap();

        assert_eq!(
            window.get_active_pane().size,
            Rect {
                row: 0,
                col: 0,
                height: 0,
                width: 0
            }
        );
    }
}
