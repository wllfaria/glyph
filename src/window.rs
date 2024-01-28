use crossterm::{
    cursor,
    style::{Color, Print, Stylize},
    terminal::Clear,
    QueueableCommand,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{stdout, Result, Stdout, Write},
    rc::Rc,
};

use crate::{
    pane::{Pane, Position},
    state::State,
};

const NO_PANE_ATTACHED: &str = "No pane attached to window";

#[derive(Debug)]
pub struct Window {
    pub id: u16,
    panes: HashMap<u16, Rc<RefCell<Pane>>>,
    last_id: u16,
    total_panes: u16,
    height: u16,
    width: u16,
    active_pane: Option<Rc<RefCell<Pane>>>,
    stdout: Stdout,
    state: Rc<RefCell<State>>,
}

impl Window {
    pub fn new(id: u16, state: Rc<RefCell<State>>) -> Result<Self> {
        let (width, height) = crossterm::terminal::size()?;
        Ok(Self {
            id,
            panes: HashMap::new(),
            stdout: stdout(),
            last_id: 0,
            total_panes: 0,
            height,
            width,
            active_pane: None,
            state,
        })
    }

    pub fn attach_pane(&mut self, pane: Rc<RefCell<Pane>>) {
        pane.borrow_mut()
            .set_pane_position(0, 0, self.height, self.width);
        self.add_pane(pane.clone());
        self.active_pane = Some(pane.clone())
    }

    pub fn render(&mut self) -> Result<()> {
        self.clear()?;
        self.render_status_bar()?;
        for pane in self.panes.values() {
            pane.borrow_mut().render()?;
        }
        self.stdout.flush()?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.stdout
            .queue(Clear(crossterm::terminal::ClearType::All))?;
        Ok(())
    }

    fn render_status_bar(&mut self) -> Result<()> {
        let pane = self.active_pane.as_ref().expect(NO_PANE_ATTACHED).borrow();
        let offset = pane.cursor_left_limit;
        let Position { x, y } = pane.cursor;
        let col_and_row = (y + 1).to_string() + ":" + &(x.saturating_sub(offset)).to_string();

        self.stdout
            .queue(cursor::MoveTo(
                self.width - 11 - col_and_row.len() as u16,
                self.height - 2,
            ))?
            .queue(Print(col_and_row.with(Color::Blue)))?
            .queue(cursor::MoveTo(self.width - 9, self.height - 2))?
            .queue(Print("17:51:45"))?;
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
            pane_mut.set_pane_position(0, i as u16 * width, self.height, width);
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
        let state = Rc::new(RefCell::new(State::new()));
        let buffer = Arc::new(Mutex::new(Buffer::new(1, None)));
        let mut pane_one = Pane::new(1, state.clone());
        let mut pane_two = Pane::new(2, state.clone());
        let mut window = Window::new(1, state.clone()).unwrap();

        pane_one.attach_buffer(buffer.clone());
        pane_two.attach_buffer(buffer.clone());

        window.width = 50;
        window.height = 30;
        window.attach_pane(Rc::new(RefCell::new(pane_one)));
        window.split_vertical(Rc::new(RefCell::new(pane_two)));

        let pane_one = window.panes.get(&1).unwrap();
        let pane_two = window.panes.get(&2).unwrap();
        let pane_one_ref = pane_one.borrow();
        let pane_two_ref = pane_two.borrow();

        assert_eq!(pane_one_ref.width, 25);
        assert_eq!(pane_two_ref.width, 25);
    }
}
