use std::collections::HashMap;

use crate::config::KeyAction;
use crate::editor::Mode;
use crate::lsp::IncomingMessage;
use crate::pane::Pane;
use crate::tui::rect::Rect;

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
