use std::{
    cell::RefCell,
    collections::HashMap,
    io::{stdout, Result, Stdout, Write},
    rc::Rc,
};

use crate::pane::Pane;

#[derive(Debug)]
pub struct Window {
    pub panes: HashMap<u16, Rc<RefCell<Pane>>>,
    pub stdout: Stdout,
}

impl Window {
    pub fn new(pane: Rc<RefCell<Pane>>) -> Result<Self> {
        let mut pane_mut = pane.borrow_mut();
        let mut panes = HashMap::new();
        panes.insert(pane_mut.id, Rc::clone(&pane));

        let (column, rows) = crossterm::terminal::size()?;
        pane_mut.set_pane_position(0, 0, column, rows);

        let stdout = stdout();
        Ok(Self { panes, stdout })
    }

    pub fn render(&mut self) -> Result<()> {
        for pane in self.panes.values() {
            pane.borrow_mut().render()?;
        }
        self.stdout.flush()?;
        Ok(())
    }
}
