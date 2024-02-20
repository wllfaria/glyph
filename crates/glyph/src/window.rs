use std::collections::HashMap;
use std::io;

use crate::command::Command;
use crate::lsp::LspClient;
use crate::pane::{Pane, PaneSize};

pub struct Window<'a> {
    pub id: usize,
    panes: HashMap<usize, Pane<'a>>,
    active_pane: usize,
    lsp: &'a LspClient,
}

impl<'a> Window<'a> {
    pub fn new(id: usize, pane: Pane<'a>, lsp: &'a LspClient) -> Self {
        let mut panes = HashMap::new();
        let pane_id = pane.id;
        panes.insert(pane.id, pane);

        Self {
            id,
            active_pane: pane_id,
            panes,
            lsp,
        }
    }

    pub fn resize(&mut self, new_size: PaneSize) {
        for pane in self.panes.values_mut() {
            pane.resize(new_size.clone());
        }
    }

    pub fn handle(&mut self, command: Command) -> io::Result<()> {
        let active_pane = self.panes.get_mut(&self.active_pane).unwrap();
        match command {
            Command::Pane(_) => active_pane.handle(command)?,
            Command::Buffer(_) => active_pane.handle(command)?,
            Command::Cursor(_) => active_pane.handle(command)?,
            Command::Window(_) => (),
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
