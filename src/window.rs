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
    pane::{Pane, PaneSize, Position},
    view::ViewSize,
};

const NO_PANE_ATTACHED: &str = "No pane attached to window";

#[derive(Debug)]
pub struct Window {
    pub id: u16,
    panes: HashMap<u16, Rc<RefCell<Pane>>>,
    last_id: u16,
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
            last_id: 0,
            total_panes: 0,
        }
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
        let pane = self.active_pane.borrow();
        let offset = 4;
        let Position {
            render_col, row, ..
        } = pane.cursor;
        let col_and_row =
            (row + 1).to_string() + ":" + &(render_col.saturating_sub(offset)).to_string();

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

    fn add_pane(&mut self, pane: Rc<RefCell<Pane>>) {
        self.last_id += 1;
        self.total_panes += 1;
        self.panes.insert(self.last_id, pane);
    }

    pub fn split_vertical(&mut self, pane: Rc<RefCell<Pane>>) {
        self.add_pane(pane);

        for (i, pane) in self.panes.values().enumerate() {
            let mut pane_mut = pane.borrow_mut();
            let width = self.size.width / self.total_panes;
            pane_mut.resize_pane(PaneSize {
                row: 0,
                col: i as u16 * width,
                height: self.size.height,
                width,
            });
        }
    }
}
