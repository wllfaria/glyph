use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::rc::Rc;

use crate::command::Command;
use crate::pane::Pane;

#[derive(Debug)]
pub struct Window {
    pub id: u16,
    panes: HashMap<u16, Rc<RefCell<Pane>>>,
    active_pane: Rc<RefCell<Pane>>,
}

impl Window {
    pub fn new(id: u16, pane: Rc<RefCell<Pane>>) -> Self {
        let mut panes = HashMap::new();
        panes.insert(pane.borrow().id, pane.clone());
        Self {
            id,
            panes,
            active_pane: pane.clone(),
        }
    }

    pub fn handle(&self, command: Command) -> io::Result<()> {
        match command {
            Command::Pane(_) => self.active_pane.borrow_mut().handle(command)?,
            Command::Buffer(_) => self.active_pane.borrow_mut().handle(command)?,
            Command::Cursor(_) => self.active_pane.borrow_mut().handle(command)?,
            Command::Window(_) => (),
            _ => {}
        }
        Ok(())
    }

    pub fn initialize(&mut self) -> io::Result<()> {
        self.render_panes()?;
        Ok(())
    }

    pub fn get_active_pane(&self) -> Rc<RefCell<Pane>> {
        self.active_pane.clone()
    }

    fn render_panes(&mut self) -> io::Result<()> {
        for pane in self.panes.values() {
            pane.borrow_mut().initialize()?;
        }
        Ok(())
    }
}
