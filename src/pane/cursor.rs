use crate::command::{Command, CursorCommands};
use crate::pane::Position;

#[derive(Debug)]
pub struct Cursor {
    pub position: Position,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            position: Position::default(),
        }
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

    pub fn get_readable_position(&self) -> (u16, u16) {
        (self.position.col + 1, self.position.row + 1)
    }

    fn move_up(&mut self, lines: &[String]) {
        self.position.row = self.position.row.saturating_sub(1);
        let line_len = lines[self.position.row as usize].len() as u16;
        match self.position.col {
            col if col > 0 && self.position.row == 0 => self.position.col = 0,
            col if col > line_len => self.position.col = line_len,
            _ => (),
        }
    }

    fn move_right(&mut self, lines: &[String]) {
        let total_lines = lines.len() as u16 - 1;
        let line_len = lines[self.position.row as usize].len() as u16;
        match self.position.col {
            col if col >= line_len && self.position.row == total_lines => {
                self.position.col = line_len
            }
            col if col >= line_len => {
                self.position.row += 1;
                self.position.col = 0;
            }
            _ => self.position.col += 1,
        }
    }

    fn move_down(&mut self, lines: &[String]) {
        let total_lines = lines.len() as u16 - 1;
        let line_len = lines[self.position.row as usize].len() as u16;
        match self.position.row {
            row if row < total_lines && self.position.col > line_len => {
                self.position.col = line_len;
                self.position.row += 1;
            }
            row if row == total_lines => self.position.col = line_len,
            _ => self.position.row = std::cmp::min(self.position.row + 1, total_lines),
        }
    }

    fn move_left(&mut self, lines: &[String]) {
        match self.position.col {
            col if col == 0 && self.position.row == 0 => self.position.col = 0,
            col if col == 0 && self.position.row > 0 => {
                self.position.row -= 1;
                let line_len = lines[self.position.row as usize].len() as u16;
                self.position.col = line_len;
            }
            _ => self.position.col = self.position.col.saturating_sub(1),
        }
    }
}
