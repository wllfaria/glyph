use std::io::Result;

use crate::command::{Command, CursorCommands};

#[derive(Debug)]
pub struct Cursor {
    pub row: u16,
    pub col: u16,
}

impl Cursor {
    pub fn new() -> Self {
        Self { row: 0, col: 0 }
    }

    pub fn handle(&mut self, command: &Command) {
        match command {
            Command::Cursor(CursorCommands::MoveUp) => self.move_up(),
            Command::Cursor(CursorCommands::MoveRight) => self.move_right(),
            Command::Cursor(CursorCommands::MoveDown) => self.move_down(),
            Command::Cursor(CursorCommands::MoveLeft) => self.move_left(),
            _ => (),
        }
    }

    fn move_up(&mut self) {
        self.row = self.row.saturating_sub(1);
    }

    fn move_right(&mut self) {
        self.col += 1;
    }

    fn move_down(&mut self) {
        self.row += 1;
    }

    fn move_left(&mut self) {
        self.col = self.col.saturating_sub(1);
    }
}
