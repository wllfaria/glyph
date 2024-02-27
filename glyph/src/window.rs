use std::collections::HashMap;
use std::io;

use crate::config::KeyAction;
use crate::editor::Mode;
use crate::lsp::IncomingMessage;
use crate::pane::{Pane, PaneSize};

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

    pub fn resize(&mut self, new_size: PaneSize) -> anyhow::Result<()> {
        for pane in self.panes.values_mut() {
            pane.resize(new_size.clone())?;
        }
        Ok(())
    }

    pub fn handle_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        let active_pane = self.panes.get_mut(&self.active_pane).unwrap();
        match action {
            KeyAction::Simple(_) => active_pane.handle_action(action, mode)?,
            _ => {}
        }
        Ok(())
    }

    pub fn handle_lsp_message(&mut self, message: (IncomingMessage, Option<String>)) {
        let active_pane = self.panes.get_mut(&self.active_pane).unwrap();
        active_pane.handle_lsp_message(message);
    }

    pub fn initialize(&mut self, mode: &Mode) -> io::Result<()> {
        self.render_panes(mode)?;
        Ok(())
    }

    pub fn get_active_pane(&self) -> &Pane {
        self.panes.get(&self.active_pane).unwrap()
    }

    fn render_panes(&mut self, mode: &Mode) -> io::Result<()> {
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
    use crate::config::*;
    use crate::theme::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_resizing() {
        let buffer = Buffer::new(1, None).unwrap();
        let theme = Theme::default();
        let config = Config::default();
        let pane = Pane::new(1, Rc::new(RefCell::new(buffer)), &theme, &config);
        let mut window = Window::new(1, pane);

        assert_eq!(
            window.get_active_pane().size,
            PaneSize {
                row: 0,
                col: 0,
                height: 0,
                width: 0
            }
        );

        window.resize((10, 10).into());

        assert_eq!(
            window.get_active_pane().size,
            PaneSize {
                row: 0,
                col: 0,
                height: 10,
                width: 10
            }
        );
    }
}
