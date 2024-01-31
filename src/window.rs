use crossterm::{
    cursor,
    style::{Color, Print, Stylize},
    QueueableCommand,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{stdout, Result, Stdout},
    rc::Rc,
};

use crate::{
    command::{Command, EditorCommands, WindowCommands},
    pane::{Pane, PaneDimensions},
    view::ViewSize,
};

#[derive(Debug)]
pub struct Window {
    pub id: u16,
    panes: HashMap<u16, Rc<RefCell<Pane>>>,
    total_panes: u16,
    active_pane: Rc<RefCell<Pane>>,
    stdout: Stdout,
    size: ViewSize,
}

impl Window {
    pub fn new(id: u16, size: ViewSize, pane: Rc<RefCell<Pane>>) -> Self {
        let mut panes = HashMap::new();
        panes.insert(pane.borrow().id, pane.clone());
        Self {
            id,
            size,
            panes,
            active_pane: pane.clone(),
            stdout: stdout(),
            total_panes: 0,
        }
    }

    pub fn handle(&self, command: Command) -> Result<()> {
        match command {
            Command::Pane(_) => self.active_pane.borrow_mut().handle(command)?,
            Command::Buffer(_) => self.active_pane.borrow_mut().handle(command)?,
            Command::Cursor(_) => self.active_pane.borrow_mut().handle(command)?,
            Command::Window(_) => (),
            _ => {}
        }
        Ok(())
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.render_panes()?;
        Ok(())
    }

    fn render_panes(&mut self) -> Result<()> {
        for pane in self.panes.values() {
            pane.borrow_mut().initialize()?;
        }
        Ok(())
    }
}
