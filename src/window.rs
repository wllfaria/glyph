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
    command::Command,
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

    pub fn handle_command(&self, command: Command) {
        match command {
            Command::Pane(_) => self.active_pane.borrow_mut().handle_command(command),
            Command::Buffer(_) => self.active_pane.borrow_mut().handle_command(command),
            Command::Cursor(_) => self.active_pane.borrow_mut().handle_command(command),
            _ => {}
        }
    }

    pub fn render(&mut self) -> Result<()> {
        self.render_status_bar()?;
        for pane in self.panes.values() {
            pane.borrow_mut().render()?;
        }
        Ok(())
    }

    fn render_status_bar(&mut self) -> Result<()> {
        let pane = self.active_pane.borrow();
        let offset = 4;
        let row = self.active_pane.borrow().cursor.row;
        let col = self.active_pane.borrow().cursor.row;
        let col_and_row = (row + 1).to_string() + ":" + &(col.saturating_sub(offset)).to_string();

        self.stdout
            .queue(cursor::MoveTo(
                self.size.width - 11 - col_and_row.len() as u16,
                self.size.height - 2,
            ))?
            .queue(Print(col_and_row.with(Color::Blue)))?
            .queue(cursor::MoveTo(self.size.width - 9, self.size.height - 2))?
            .queue(Print("17:51:45"))?;
        Ok(())
    }

    pub fn split_vertical(&mut self, pane: Rc<RefCell<Pane>>) {
        for (i, pane) in self.panes.values().enumerate() {
            let mut pane_mut = pane.borrow_mut();
            let width = self.size.width / self.total_panes;
        }
    }
}
