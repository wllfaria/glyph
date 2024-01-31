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

    pub fn handle(&mut self, command: &Command, lines: &[String]) {
        match command {
            Command::Cursor(CursorCommands::MoveUp) => self.move_up(lines),
            Command::Cursor(CursorCommands::MoveRight) => self.move_right(lines),
            Command::Cursor(CursorCommands::MoveDown) => self.move_down(lines),
            Command::Cursor(CursorCommands::MoveLeft) => self.move_left(lines),
            _ => (),
        }
    }

    fn move_up(&mut self, lines: &[String]) {
        self.row = self.row.saturating_sub(1);
    }

    fn move_right(&mut self, lines: &[String]) {
        self.col += 1;
    }

    fn move_down(&mut self, lines: &[String]) {
        self.row += 1;
    }

    fn move_left(&mut self, lines: &[String]) {
        self.col = self.col.saturating_sub(1);
    }
}
