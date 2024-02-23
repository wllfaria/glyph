use crate::buffer::Buffer;
use crate::config::{Action, KeyAction};

use super::Position;

#[derive(Debug)]
pub struct Cursor {
    pub absolute_position: usize,
    pub row: usize,
    pub col: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            absolute_position: 0,
            row: 0,
            col: 0,
        }
    }

    pub fn handle(&mut self, action: &KeyAction, buffer: &mut Buffer) {
        match action {
            KeyAction::Simple(Action::MoveToTop) => self.move_to_top(),
            KeyAction::Simple(Action::MoveToBottom) => self.move_to_bottom(buffer),
            KeyAction::Simple(Action::MoveUp) => self.move_up(buffer),
            KeyAction::Simple(Action::MoveRight) => self.move_right(buffer),
            KeyAction::Simple(Action::MoveDown) => self.move_down(buffer),
            KeyAction::Simple(Action::MoveLeft) => self.move_left(buffer),
            KeyAction::Simple(Action::MoveToLineStart) => self.move_to_line_start(buffer),
            KeyAction::Simple(Action::MoveToLineEnd) => self.move_to_line_end(buffer),
            KeyAction::Simple(Action::NextWord) => self.move_to_next_word(buffer),
            KeyAction::Simple(Action::InsertChar(_)) => {
                self.absolute_position += 1;
                self.col += 1;
            }
            KeyAction::Simple(Action::DeletePreviousChar) => match self.col {
                c if c == 0 && self.row == 0 => (),
                0 => {
                    self.move_up(buffer);
                    self.move_to_line_end(buffer);
                }
                _ => {
                    self.col = self.col.saturating_sub(1);
                    self.absolute_position = self.absolute_position.saturating_sub(1);
                }
            },
            KeyAction::Simple(Action::InsertLine) => {
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

    pub fn move_up(&mut self, buffer: &mut Buffer) {
        if self.row == 0 {
            self.absolute_position = 0;
            self.col = 0;
            return;
        }
        if let Some(mark) = buffer.marker.get_by_line(self.row) {
            match self.col {
                0 => self.absolute_position = mark.start,
                _ if self.col > mark.size => {
                    self.absolute_position = mark.start + mark.size - 1;
                }
                _ => self.absolute_position = mark.start + self.col,
            }
            self.row = self.row.saturating_sub(1);
        }
    }

    fn move_right(&mut self, buffer: &mut Buffer) {
        if let Some(mark) = buffer.marker.get_by_line(self.row + 1) {
            self.col += 1;
            match self.col {
                col if col >= mark.size => {
                    self.col = 0;
                    self.move_down(buffer);
                }
                _ => self.absolute_position = mark.start + self.col,
            }
        }
    }

    fn move_down(&mut self, buffer: &mut Buffer) {
        let next_line = 2 + self.row;
        if let Some(mark) = buffer.marker.get_by_line(next_line) {
            self.row += 1;
            match self.col {
                0 => self.absolute_position = mark.start,
                col if col > mark.size => {
                    self.absolute_position = mark.start + mark.size.saturating_sub(1);
                }
                _ => self.absolute_position = mark.start + self.col,
            }
        } else {
            let mark = buffer.marker.get_by_line(self.row + 1).unwrap();
            self.absolute_position = mark.start + mark.size.saturating_sub(1);
            self.col = mark.size.saturating_sub(1);
        }
    }

    fn move_left(&mut self, buffer: &mut Buffer) {
        match self.col {
            0 if self.row == 0 => (),
            0 => {
                assert!(self.row > 0);
                self.move_up(buffer);
                self.move_to_line_end(buffer);
            }
            _ => {
                self.col = self.col.saturating_sub(1);
                let mark = buffer.marker.get_by_line(self.row + 1).unwrap();
                if self.col >= mark.size {
                    self.col = mark.size.saturating_sub(2);
                }
                self.absolute_position -= 1;
            }
        }
    }

    fn move_to_line_end(&mut self, buffer: &mut Buffer) {
        let Position { row, .. } = self.get_readable_position();
        let mark = buffer.marker.get_by_line(row).unwrap();
        self.col = mark.size.saturating_sub(2);
        self.absolute_position = mark.start + mark.size.saturating_sub(2);
    }

    fn move_to_top(&mut self) {
        self.row = 0;
        self.absolute_position = 0;
    }

    fn move_to_bottom(&mut self, buffer: &mut Buffer) {
        let total_lines = buffer.marker.len();
        let mark = buffer.marker.get_by_line(total_lines).unwrap();
        self.row = total_lines - 1;
        self.col = mark.size.saturating_sub(2);
        self.absolute_position = mark.start + mark.size.saturating_sub(2);
    }

    fn move_to_line_start(&mut self, buffer: &mut Buffer) {
        let mark = buffer.marker.get_by_line(self.row + 1).unwrap();
        self.col = 0;
        self.absolute_position = mark.start;
    }

    fn move_to_next_word(&mut self, buffer: &mut Buffer) {
        let content = buffer.to_string();
        let mut pos = self.absolute_position;
        while pos < content.len() && !self.is_separator(content[pos..].chars().next().unwrap()) {
            tracing::trace!("next char is not a separator");
            pos += content[pos..].chars().next().unwrap().len_utf8();
        }

        while pos < content.len() && self.is_skippable(content[pos..].chars().next().unwrap()) {
            tracing::trace!(
                "next char is a separator {}",
                content[pos..].chars().next().unwrap()
            );
            pos += content[pos..].chars().next().unwrap().len_utf8();
        }

        let mark = buffer.marker.get_by_cursor(pos).unwrap();
        let offset = pos - mark.start;
        self.col = offset;
        self.row = mark.line - 1;
        self.absolute_position = pos;
    }

    fn is_skippable(&self, c: char) -> bool {
        matches!(c, ' ' | ':')
    }

    fn is_separator(&self, c: char) -> bool {
        matches!(c, ' ' | ':' | '-' | ';' | '\n')
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

        let mark = buffer.marker.get_by_line(cursor.row + 1).unwrap();

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

        let mark = buffer.marker.get_by_line(cursor.row + 1).unwrap();

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

        assert_eq!(cursor.col, 11);
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

        assert_eq!(cursor.col, 11);
        assert_eq!(cursor.absolute_position, 11);
        assert_eq!(cursor.row, 0);
    }
}
