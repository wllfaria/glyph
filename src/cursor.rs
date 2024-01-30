use std::{cell::RefCell, rc::Rc};

use crate::command::{Command, CommandBus, CommandListener, CursorCommands};

pub struct Cursor {
    pub row: u16,
    pub col: u16,
    command_bus: Rc<RefCell<CommandBus>>,
}

impl Cursor {
    pub fn new(command_bus: Rc<RefCell<CommandBus>>) -> Self {
        Self {
            row: 0,
            col: 0,
            command_bus,
        }
    }

    pub fn setup(&mut self, cursor: Rc<RefCell<Cursor>>) {
        self.command_bus
            .borrow_mut()
            .subscribe(Box::new(CursorListener { cursor }))
    }

    fn move_left(&mut self) {
        self.col = self.col.saturating_sub(1);
    }

    fn move_right(&mut self) {
        self.col += 1;
    }

    fn move_up(&mut self) {
        self.row = self.row.saturating_sub(1);
    }

    fn move_down(&mut self) {
        self.row += 1;
    }
}

pub struct CursorListener {
    pub cursor: Rc<RefCell<Cursor>>,
}

impl CommandListener for CursorListener {
    fn call(&mut self, command: &Command, _id: u16) -> std::io::Result<()> {
        match command {
            Command::Cursor(CursorCommands::MoveLeft) => self.cursor.borrow_mut().move_left(),
            Command::Cursor(CursorCommands::MoveRight) => self.cursor.borrow_mut().move_right(),
            Command::Cursor(CursorCommands::MoveUp) => self.cursor.borrow_mut().move_up(),
            Command::Cursor(CursorCommands::MoveDown) => self.cursor.borrow_mut().move_down(),
            _ => {}
        }
        Ok(())
    }
}
