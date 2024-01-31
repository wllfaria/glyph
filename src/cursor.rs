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

    pub fn get_readable_position(&self) -> (u16, u16) {
        (self.col + 1, self.row + 1)
    }

    fn move_up(&mut self, lines: &[String]) {
        self.row = self.row.saturating_sub(1);
        let line_len = lines[self.row as usize].len() as u16;
        match self.col {
            col if col > 0 && self.row == 0 => self.col = 0,
            col if col > line_len => self.col = line_len,
            _ => (),
        }
    }

    fn move_right(&mut self, lines: &[String]) {
        let total_lines = lines.len() as u16 - 1;
        let line_len = lines[self.row as usize].len() as u16;
        match self.col {
            col if col >= line_len && self.row == total_lines => self.col = line_len,
            col if col >= line_len => {
                self.row += 1;
                self.col = 0;
            }
            _ => self.col += 1,
        }
    }

    fn move_down(&mut self, lines: &[String]) {
        let total_lines = lines.len() as u16 - 1;
        let line_len = lines[self.row as usize].len() as u16;
        match self.row {
            row if row < total_lines && self.col > line_len => {
                self.col = line_len;
                self.row += 1;
            }
            row if row == total_lines => self.col = line_len,
            _ => self.row = std::cmp::min(self.row + 1, total_lines),
        }
    }

    fn move_left(&mut self, lines: &[String]) {
        match self.col {
            col if col == 0 && self.row == 0 => self.col = 0,
            col if col == 0 && self.row > 0 => {
                self.row -= 1;
                let line_len = lines[self.row as usize].len() as u16;
                self.col = line_len;
            }
            _ => self.col = self.col.saturating_sub(1),
        }
    }
}
