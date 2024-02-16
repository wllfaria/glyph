use crate::buffer::Buffer;
use crate::command::{BufferCommands, Command, CursorCommands};
use logger;

use super::Position;

#[derive(Debug)]
pub struct Cursor {
    pub absolute_position: usize,
    pub row: u16,
    pub col: u16,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            absolute_position: 0,
            row: 0,
            col: 0,
        }
    }

    pub fn handle(&mut self, command: &Command, buffer: &mut Buffer) {
        match command {
            Command::Cursor(CursorCommands::MoveUp) => self.move_up(buffer),
            Command::Cursor(CursorCommands::MoveRight) => self.move_right(buffer),
            Command::Cursor(CursorCommands::MoveDown) => self.move_down(buffer),
            Command::Cursor(CursorCommands::MoveLeft) => self.move_left(buffer),
            Command::Buffer(BufferCommands::Type(_)) => {
                self.absolute_position += 1;
                self.col += 1;
            }
            Command::Buffer(BufferCommands::Backspace) => match self.col {
                c if c == 0 && self.row == 0 => (),
                0 => {
                    self.move_up(buffer);
                    self.move_to_end_of_line(buffer);
                }
                _ => {
                    self.col = self.col.saturating_sub(1);
                    self.absolute_position = self.absolute_position.saturating_sub(1);
                }
            },
            Command::Buffer(BufferCommands::NewLineBelow) => {
                self.absolute_position += 1;
                self.col = 0;
                self.row += 1;
            }
            _ => (),
        }
    }

    pub fn get_readable_position(&self) -> Position {
        Position {
            row: self.row + 1,
            col: self.col + 1,
        }
    }

    fn move_up(&mut self, buffer: &mut Buffer) {
        if self.row == 0 {
            self.absolute_position = 0;
            self.col = 0;
            return;
        }
        if let Some(mark) = buffer.marker.get_by_line(self.row as usize) {
            match self.col {
                0 => self.absolute_position = mark.start,
                _ if self.col as usize > mark.size => {
                    self.absolute_position = mark.start + mark.size - 1;
                }
                _ => self.absolute_position = mark.start + self.col as usize,
            }
            self.row = self.row.saturating_sub(1);
        }
    }

    fn move_right(&mut self, buffer: &mut Buffer) {
        if let Some(mark) = buffer.marker.get_by_line(self.row as usize + 1) {
            self.col += 1;
            match self.col {
                col if col as usize >= mark.size => {
                    self.col = 0;
                    self.move_down(buffer);
                }
                _ => self.absolute_position = mark.start + self.col as usize,
            }
        }
    }

    fn move_down(&mut self, buffer: &mut Buffer) {
        let next_line = 2 + self.row as usize;
        if let Some(mark) = buffer.marker.get_by_line(next_line) {
            self.row += 1;
            match self.col {
                0 => self.absolute_position = mark.start,
                col if col as usize > mark.size => {
                    self.absolute_position = mark.start + mark.size.saturating_sub(1);
                }
                _ => self.absolute_position = mark.start + self.col as usize,
            }
        } else {
            let mark = buffer.marker.get_by_line(self.row as usize + 1).unwrap();
            self.absolute_position = mark.start + mark.size.saturating_sub(1);
            self.col = mark.size.saturating_sub(1) as u16;
        }
    }

    fn move_left(&mut self, buffer: &mut Buffer) {
        match self.col {
            0 if self.row == 0 => (),
            0 => {
                assert!(self.row > 0);
                self.move_up(buffer);
                self.move_to_end_of_line(buffer);
            }
            _ => {
                self.col = self.col.saturating_sub(1);
                let mark = buffer.marker.get_by_line(self.row as usize + 1).unwrap();
                if self.col as usize >= mark.size {
                    self.col = mark.size.saturating_sub(2) as u16;
                }
                self.absolute_position -= 1;
            }
        }
    }

    fn move_to_end_of_line(&mut self, buffer: &mut Buffer) {
        let mark = buffer.marker.get_by_line(self.row as usize + 1).unwrap();
        self.col = mark.size.saturating_sub(1) as u16;
        self.absolute_position = mark.start + mark.size.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cursor_move_up() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello\nWorld\nEveryone", gap);
        cursor.row = 1;

        cursor.move_up(&mut buffer);

        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.absolute_position, 0);
        assert_eq!(buffer.buffer[cursor.absolute_position + gap], 'H');
    }

    #[test]
    fn test_cursor_move_down() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello\nWorld\nEveryone", gap);
        cursor.row = 0;

        cursor.move_down(&mut buffer);

        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.absolute_position, 6);
        assert_eq!(buffer.buffer[cursor.absolute_position + gap], 'W');
    }

    #[test]
    fn test_cursor_move_right() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello\nWorld\nEveryone", gap);

        cursor.move_right(&mut buffer);

        assert_eq!(cursor.col, 1);
        assert_eq!(cursor.absolute_position, 1);
        assert_eq!(buffer.buffer[cursor.absolute_position + gap], 'e');
    }

    #[test]
    fn test_cursor_move_left() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello\nWorld\nEveryone", gap);
        cursor.col = 1;
        cursor.absolute_position = 1;

        cursor.move_left(&mut buffer);

        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.absolute_position, 0);
        assert_eq!(buffer.buffer[cursor.absolute_position + gap], 'H');
    }

    #[test]
    fn test_moving_down_into_shorter_line() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer =
            Buffer::from_string(1, "Hello World! This is a big line\n this isn't", gap);

        for _ in 0..20 {
            cursor.move_right(&mut buffer);
        }

        assert_eq!(cursor.col, 20);
        assert_eq!(cursor.absolute_position, 20);
        cursor.move_down(&mut buffer);

        let mark = buffer
            .marker
            .get_by_cursor(cursor.absolute_position)
            .unwrap();

        assert_eq!(cursor.col, 20);
        assert_eq!(cursor.absolute_position, mark.start + mark.size - 1);
        assert_eq!(buffer.buffer[cursor.absolute_position + gap], 't');
    }

    #[test]
    fn test_moving_up_into_shorter_line() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello\nWorld! This is a big line", gap);
        cursor.move_down(&mut buffer);
        cursor.col = 20;
        cursor.absolute_position += 20;

        assert_eq!(cursor.col, 20);
        assert_eq!(cursor.absolute_position, 26);

        cursor.move_up(&mut buffer);

        let mark = buffer
            .marker
            .get_by_cursor(cursor.absolute_position)
            .unwrap();

        assert_eq!(cursor.col, 20);
        assert_eq!(cursor.absolute_position, mark.start + mark.size - 1);
        assert_eq!(buffer.buffer[cursor.absolute_position + gap], '\n');
    }

    #[test]
    fn test_moving_up_into_longer_line() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello World! This is a big line\nThis isn't", gap);
        cursor.move_down(&mut buffer);
        cursor.col = 5;
        cursor.absolute_position += 5;

        assert_eq!(cursor.col, 5);
        assert_eq!(cursor.absolute_position, 37);

        cursor.move_up(&mut buffer);

        let mark = buffer
            .marker
            .get_by_cursor(cursor.absolute_position)
            .unwrap();

        assert_eq!(cursor.col, 5);
        assert_eq!(cursor.absolute_position, 5);
        assert_eq!(mark.start, 0);
        assert_eq!(buffer.buffer[cursor.absolute_position + gap], ' ');
    }

    #[test]
    fn test_moving_down_into_longer_line() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello\nWorld! This is a big line", gap);
        cursor.row = 0;
        cursor.col = 5;
        cursor.absolute_position = 5;

        cursor.move_down(&mut buffer);

        let mark = buffer
            .marker
            .get_by_cursor(cursor.absolute_position)
            .unwrap();

        assert_eq!(cursor.col, 5);
        assert_eq!(cursor.absolute_position, 11);
        assert_eq!(mark.start, 6);
        assert_eq!(buffer.buffer[cursor.absolute_position + gap], '!');
    }

    #[test]
    fn test_moving_right_into_line_below() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello\nWorld!", gap);

        for _ in 0..6 {
            cursor.move_right(&mut buffer);
        }

        let mark = buffer.marker.get_by_line(cursor.row as usize + 1).unwrap();

        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.absolute_position, mark.start);
        assert_eq!(cursor.row, 1);
    }

    #[test]
    fn test_moving_left_into_line_above() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello\nWorld!", gap);
        cursor.move_down(&mut buffer);

        cursor.move_left(&mut buffer);

        let mark = buffer.marker.get_by_line(cursor.row as usize + 1).unwrap();

        assert_eq!(cursor.col, 5);
        assert_eq!(cursor.absolute_position, mark.size - 1);
        assert_eq!(cursor.row, 0);
    }

    #[test]
    fn test_should_not_go_left_when_at_start_of_file() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello\nWorld!", gap);

        cursor.move_left(&mut buffer);

        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.absolute_position, 0);
        assert_eq!(cursor.row, 0);
    }

    #[test]
    fn test_should_go_to_line_start_when_moving_up_from_start_of_file() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello World!", gap);
        cursor.absolute_position = 5;
        cursor.col = 5;

        cursor.move_up(&mut buffer);

        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.absolute_position, 0);
        assert_eq!(cursor.row, 0);
    }

    #[test]
    fn test_should_go_to_line_end_when_moving_down_from_end_of_file() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello World!", gap);

        cursor.move_down(&mut buffer);

        assert_eq!(cursor.col, 12);
        assert_eq!(cursor.absolute_position, 11);
        assert_eq!(cursor.row, 0);
    }

    #[test]
    fn test_should_not_go_right_when_at_end_of_file() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello World!", gap);
        cursor.absolute_position = 11;
        cursor.col = 11;

        cursor.move_right(&mut buffer);

        assert_eq!(cursor.col, 12);
        assert_eq!(cursor.absolute_position, 11);
        assert_eq!(cursor.row, 0);
    }
}
