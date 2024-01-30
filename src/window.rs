use crossterm::{terminal::Clear, QueueableCommand};
use std::{
    cell::RefCell,
    io::{stdout, Result, Stdout, Write},
    rc::Rc,
};

use crate::{
    command::{Command, CommandBus, CommandListener, EditorCommands},
    pane::{Pane, PaneSize},
};

pub struct Window {
    pub id: u16,
    size: (u16, u16),
    stdout: Stdout,
    panes: Vec<Rc<RefCell<Pane>>>,
    command_bus: Rc<RefCell<CommandBus>>,
}

impl Window {
    pub fn new(id: u16, command_bus: Rc<RefCell<CommandBus>>, pane: Rc<RefCell<Pane>>) -> Self {
        Self {
            id,
            stdout: stdout(),
            panes: vec![pane],
            command_bus,
            size: (0, 0),
        }
    }

    pub fn render(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn setup(&mut self, window: Rc<RefCell<Window>>) {
        self.size = crossterm::terminal::size().unwrap();
        self.resize_panes();
        self.command_bus
            .borrow_mut()
            .subscribe(Box::new(WindowListener { window }));
    }

    fn resize_panes(&self) {
        for (i, pane) in self.panes.iter().enumerate() {
            let mut pane_mut = pane.borrow_mut();
            let width = self.size.0 / self.panes.len() as u16;
            pane_mut.resize_pane(PaneSize {
                row: 0,
                col: i as u16 * width,
                height: self.size.1,
                width,
            });
        }
    }
}

pub struct WindowListener {
    pub window: Rc<RefCell<Window>>,
}

impl CommandListener for WindowListener {
    fn call(&mut self, command: &Command, _id: u16) -> Result<()> {
        if let Command::Editor(EditorCommands::Render) = command {
            self.window.borrow_mut().render()?;
        }
        Ok(())
    }
}
