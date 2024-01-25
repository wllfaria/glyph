use std::{
    cell::RefCell,
    collections::HashMap,
    io::{stdout, Result, Stdout, Write},
    rc::Rc,
};

use crossterm::{terminal::Clear, QueueableCommand};

use crate::pane::Pane;

#[derive(Debug)]
pub struct Window {
    last_id: u16,
    total_panes: u16,
    height: u16,
    width: u16,
    pub panes: HashMap<u16, Rc<RefCell<Pane>>>,
    pub stdout: Stdout,
}

impl Window {
    pub fn new(pane: Rc<RefCell<Pane>>) -> Result<Self> {
        let mut pane_mut = pane.borrow_mut();
        let mut panes = HashMap::new();

        panes.insert(pane_mut.id, Rc::clone(&pane));
        let (width, height) = crossterm::terminal::size()?;
        pane_mut.set_pane_position(0, 0, height, width);
        Ok(Self {
            panes,
            stdout: stdout(),
            last_id: pane_mut.id,
            total_panes: 1,
            height,
            width,
        })
    }

    pub fn render(&mut self) -> Result<()> {
        self.stdout
            .queue(Clear(crossterm::terminal::ClearType::All))?;
        for pane in self.panes.values() {
            pane.borrow_mut().render()?;
            self.stdout.flush()?;
        }
        Ok(())
    }

    fn add_pane(&mut self, pane: Rc<RefCell<Pane>>) {
        self.last_id += 1;
        self.total_panes += 1;
        self.panes.insert(self.last_id, pane);
    }

    pub fn split_vertical(&mut self, pane: Rc<RefCell<Pane>>) {
        self.add_pane(pane);

        for (i, pane) in self.panes.values().enumerate() {
            let mut pane_mut = pane.borrow_mut();
            let width = self.width / self.total_panes;

            pane_mut.col = i as u16 * width;
            pane_mut.width = width;
            pane_mut.height = self.height;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;
    use crate::pane::Pane;
    use std::{
        cell::RefCell,
        rc::Rc,
        sync::{Arc, Mutex},
    };

    #[test]
    pub fn should_split_vertically() {
        let buffer = Arc::new(Mutex::new(Buffer::new(None)));
        let pane_one = Pane::new(0, Arc::clone(&buffer));
        let pane_two = Pane::new(0, Arc::clone(&buffer));
        let mut window = Window::new(Rc::new(RefCell::new(pane_one))).unwrap();
        window.width = 50;
        window.height = 30;

        window.split_vertical(Rc::new(RefCell::new(pane_two)));

        let pane_one = window.panes.get(&0).unwrap();
        let pane_two = window.panes.get(&1).unwrap();
        let pane_one_ref = pane_one.borrow();
        let pane_two_ref = pane_two.borrow();

        assert_eq!(pane_one_ref.width, 25);
        assert_eq!(pane_two_ref.width, 25);
    }
}
