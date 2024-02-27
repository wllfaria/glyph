use crate::buffer::Buffer;
use crate::config::{Action, KeyAction};
use crate::editor::Mode;

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

    pub fn handle(&mut self, action: &KeyAction, buffer: &mut Buffer, mode: &Mode) {
        match action {
            KeyAction::Simple(Action::MoveToTop) => self.move_to_top(),
            KeyAction::Simple(Action::MoveToBottom) => self.move_to_bottom(buffer),
            KeyAction::Simple(Action::MoveUp) => self.move_up(buffer),
            KeyAction::Simple(Action::MoveRight) => self.move_right(buffer, mode),
            KeyAction::Simple(Action::MoveDown) => self.move_down(buffer, mode),
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
            KeyAction::Simple(Action::InsertLineBelow) => {
                self.col = 0;
                self.row += 1;
                if let Some(mark) = buffer.marker.get_by_line(self.row + 1) {
                    self.absolute_position = mark.start;
                }
            }
            KeyAction::Simple(Action::InsertLineAbove) => {
                self.col = 0;
                if let Some(mark) = buffer.marker.get_by_line(self.row + 1) {
                    self.absolute_position = mark.start;
                }
            }
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
        match (self.col, self.row) {
            (_, 0) => {
                self.absolute_position = 0;
                self.col = 0;
            }
            (0, _) => {
                let mark = buffer
                    .marker
                    .get_by_line(self.row)
                    .expect("if we are above line 0, there sould always be a line above");
                self.absolute_position = mark.start;
                self.row -= 1;
            }
            (x, _) => {
                let mark = buffer
                    .marker
                    .get_by_line(self.row)
                    .expect("if we are above line 0, there sould always be a line above");
                self.absolute_position = match x > mark.size.saturating_sub(2) {
                    true => mark.start + mark.size.saturating_sub(2),
                    false => mark.start + self.col,
                };
                self.row -= 1;
            }
        }
    }

    fn insert_right(&mut self, _: &mut Buffer) {
        self.col += 1;
        self.absolute_position += 1;
    }

    fn move_right(&mut self, buffer: &mut Buffer, mode: &Mode) {
        if let Some(mark) = buffer.marker.get_by_line(self.row + 1) {
            let limit = match mode {
                Mode::Normal => mark.size.saturating_sub(2),
                _ => mark.size.saturating_sub(1),
            };
            self.col = usize::min(self.col + 1, limit);
            self.absolute_position = mark.start + self.col;
        }
    }

    fn move_down(&mut self, buffer: &mut Buffer, mode: &Mode) {
        let next_line = 2 + self.row;
        match buffer.marker.get_by_line(next_line) {
            Some(mark) => {
                self.row += 1;
                let limit = match mode {
                    Mode::Normal => mark.size.saturating_sub(2),
                    _ => mark.size.saturating_sub(1),
                };

                match self.col > limit {
                    true => self.absolute_position = mark.start + limit,
                    false => self.absolute_position = mark.start + self.col,
                }
            }
            None => {
                let mark = buffer
                    .marker
                    .get_by_line(self.row + 1)
                    .expect("current line should never be none");
                let limit = match mode {
                    Mode::Normal => mark.size.saturating_sub(2),
                    _ => mark.size.saturating_sub(1),
                };
                self.col = limit;
                self.absolute_position = mark.start + limit;
            }
        }
    }

    fn move_left(&mut self, _: &mut Buffer) {
        if self.col > 0 {
            self.col -= 1;
            self.absolute_position -= 1;
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
        self.col = 0;
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

    // FIXME: this is not behaving as vim would, currently we are not as
    // smart as vim W
    fn move_to_next_word(&mut self, buffer: &mut Buffer) {
        let content = buffer.to_string();
        let mut pos = self.absolute_position;

        let starting_char = content
            .chars()
            .nth(pos)
            .expect("current position should never be out of bounds");
        let starting_on_separator = self.is_separator(starting_char);

        let mut iter = content[pos..].chars();
        while let Some(char) = iter.next() {
            if starting_on_separator {
                // we should skip repeated separators, such as :: or //
                if !char.is_whitespace() && char != starting_char {
                    break;
                }
                pos += 1;
                continue;
            }

            // if its not a separator, we just skip it
            if !self.is_separator(char) {
                pos += 1;
                continue;
            }

            // if it is a separator, and a whitespace, we skip until the next non whitespace
            if char.is_whitespace() {
                while let Some(c) = content[pos..].chars().nth(0) {
                    if !c.is_whitespace() {
                        break;
                    }
                    pos += 1;
                }
            }
            break;
        }

        if let Some(mark) = buffer.marker.get_by_cursor(pos) {
            let offset = pos - mark.start;
            self.col = offset;
            self.row = mark.line - 1;
            self.absolute_position = pos;
        }
    }

    fn is_separator(&self, c: char) -> bool {
        matches!(c, ' ' | ':' | '-' | '}' | ')' | ']' | ',' | '(' | '>')
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

        cursor.handle(
            &KeyAction::Simple(Action::MoveUp),
            &mut buffer,
            &Mode::Normal,
        );

        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.absolute_position, 0);
        assert_eq!(buffer.buffer[cursor.absolute_position + gap], 'H');
    }

    #[test]
    fn test_get_readable_position() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello\nWorld\nEveryone", gap);
        cursor.row = 1;

        let pos = cursor.get_readable_position();

        assert_eq!(pos.col, 1);
        assert_eq!(pos.row, 2);
    }

    #[test]
    fn move_to_line_start() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer =
            Buffer::from_string(1, "Hello World! This is a big line\n this isn't\n", gap);

        for _ in 0..20 {
            cursor.handle(
                &KeyAction::Simple(Action::MoveRight),
                &mut buffer,
                &Mode::Normal,
            );
        }

        cursor.handle(
            &KeyAction::Simple(Action::MoveToLineStart),
            &mut buffer,
            &Mode::Normal,
        );

        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.absolute_position, 0);
    }

    #[test]
    fn move_to_line_end() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer =
            Buffer::from_string(1, "Hello World! This is a big line\n this isn't\n", gap);

        cursor.handle(
            &KeyAction::Simple(Action::MoveToLineEnd),
            &mut buffer,
            &Mode::Normal,
        );

        assert_eq!(cursor.col, 30);
        assert_eq!(cursor.absolute_position, 30);
    }

    #[test]
    fn test_cursor_move_down() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello\nWorld\nEveryone", gap);
        cursor.row = 0;

        cursor.handle(
            &KeyAction::Simple(Action::MoveDown),
            &mut buffer,
            &Mode::Normal,
        );

        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.absolute_position, 6);
        assert_eq!(buffer.buffer[cursor.absolute_position + gap], 'W');
    }

    #[test]
    fn test_cursor_move_right() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello\nWorld\nEveryone", gap);

        cursor.handle(
            &KeyAction::Simple(Action::MoveRight),
            &mut buffer,
            &Mode::Normal,
        );

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

        cursor.handle(
            &KeyAction::Simple(Action::MoveLeft),
            &mut buffer,
            &Mode::Normal,
        );

        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.absolute_position, 0);
        assert_eq!(buffer.buffer[cursor.absolute_position + gap], 'H');
    }

    #[test]
    fn test_moving_down_into_shorter_line() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer =
            Buffer::from_string(1, "Hello World! This is a big line\n this isn't\n", gap);

        for _ in 0..20 {
            cursor.handle(
                &KeyAction::Simple(Action::MoveRight),
                &mut buffer,
                &Mode::Normal,
            );
        }

        assert_eq!(cursor.col, 20);
        assert_eq!(cursor.absolute_position, 20);
        cursor.handle(
            &KeyAction::Simple(Action::MoveDown),
            &mut buffer,
            &Mode::Normal,
        );

        let mark = buffer
            .marker
            .get_by_cursor(cursor.absolute_position)
            .unwrap();

        assert_eq!(cursor.col, 20);
        assert_eq!(cursor.absolute_position, mark.start + mark.size - 2);
        assert_eq!(buffer.buffer[cursor.absolute_position + gap], 't');
    }

    #[test]
    fn test_moving_up_into_shorter_line() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer =
            Buffer::from_string(1, "Hello\nWorld! This is a big line we got here", gap);
        cursor.handle(
            &KeyAction::Simple(Action::MoveDown),
            &mut buffer,
            &Mode::Normal,
        );
        cursor.col = 20;
        cursor.absolute_position += 20;

        assert_eq!(cursor.col, 20);
        assert_eq!(cursor.absolute_position, 26);

        cursor.handle(
            &KeyAction::Simple(Action::MoveUp),
            &mut buffer,
            &Mode::Normal,
        );

        let mark = buffer
            .marker
            .get_by_cursor(cursor.absolute_position)
            .unwrap();

        assert_eq!(cursor.col, 20);
        assert_eq!(cursor.absolute_position, mark.start + mark.size - 2);
        assert_eq!(buffer.buffer[cursor.absolute_position + gap], 'o');
    }

    #[test]
    fn test_moving_up_into_longer_line() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello World! This is a big line\nThis isn't", gap);
        cursor.handle(
            &KeyAction::Simple(Action::MoveDown),
            &mut buffer,
            &Mode::Normal,
        );
        cursor.col = 5;
        cursor.absolute_position += 5;

        assert_eq!(cursor.col, 5);
        assert_eq!(cursor.absolute_position, 37);

        cursor.handle(
            &KeyAction::Simple(Action::MoveUp),
            &mut buffer,
            &Mode::Normal,
        );

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

        cursor.handle(
            &KeyAction::Simple(Action::MoveDown),
            &mut buffer,
            &Mode::Normal,
        );

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
    fn test_should_not_go_left_when_at_start_of_file() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello\nWorld!", gap);

        cursor.handle(
            &KeyAction::Simple(Action::MoveLeft),
            &mut buffer,
            &Mode::Normal,
        );

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

        cursor.handle(
            &KeyAction::Simple(Action::MoveUp),
            &mut buffer,
            &Mode::Normal,
        );

        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.absolute_position, 0);
        assert_eq!(cursor.row, 0);
    }

    #[test]
    fn test_should_go_to_line_end_when_moving_down_from_end_of_file() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello World!", gap);

        cursor.handle(
            &KeyAction::Simple(Action::MoveDown),
            &mut buffer,
            &Mode::Normal,
        );

        assert_eq!(cursor.col, 10);
        assert_eq!(cursor.absolute_position, 10);
        assert_eq!(cursor.row, 0);
    }

    #[test]
    fn test_should_not_go_right_when_at_end_of_file() {
        let gap = 5;
        let mut cursor = Cursor::new();
        let mut buffer = Buffer::from_string(1, "Hello World!", gap);
        cursor.absolute_position = 11;
        cursor.col = 11;

        cursor.handle(
            &KeyAction::Simple(Action::MoveRight),
            &mut buffer,
            &Mode::Normal,
        );

        assert_eq!(cursor.col, 10);
        assert_eq!(cursor.absolute_position, 10);
        assert_eq!(cursor.row, 0);
    }
}
