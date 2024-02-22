use std::collections::HashMap;
use std::io;

use crate::config::KeyAction;
use crate::pane::{Pane, PaneSize};

pub struct Window<'a> {
    pub id: usize,
    panes: HashMap<usize, Pane<'a>>,
    active_pane: usize,
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

    pub fn resize(&mut self, new_size: PaneSize) {
        for pane in self.panes.values_mut() {
            pane.resize(new_size.clone());
        }
    }

    pub fn handle_action(&mut self, action: &KeyAction) -> io::Result<()> {
        let active_pane = self.panes.get_mut(&self.active_pane).unwrap();
        match action {
            KeyAction::Simple(_) => active_pane.handle_action(action)?,
            _ => {}
        }
        Ok(())
    }

    pub fn initialize(&mut self) -> io::Result<()> {
        self.render_panes()?;
        Ok(())
    }

    pub fn get_active_pane(&self) -> &Pane {
        self.panes.get(&self.active_pane).unwrap()
    }

    fn render_panes(&mut self) -> io::Result<()> {
        for pane in self.panes.values_mut() {
            pane.initialize()?;
        }
        Ok(())
    }
}
