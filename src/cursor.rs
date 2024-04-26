use std::ops::Add;

use crate::buffer::TextObject;
use crate::config::{Action, KeyAction};
use crate::editor::Mode;
use crate::tui::position::Position;

#[derive(Debug, Default)]
pub struct Cursor {
    pub absolute_position: usize,
    pub row: usize,
    pub col: usize,
}

impl Cursor {
    pub fn handle(&mut self, action: &KeyAction, buffer: &mut TextObject, mode: &Mode) {
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
            KeyAction::Simple(Action::InsertChar(_)) => self.insert_char(),
            KeyAction::Simple(Action::DeletePreviousChar) => self.delete_prev_char(buffer),
            KeyAction::Simple(Action::InsertLineBelow) => self.insert_line_below(buffer),
            KeyAction::Simple(Action::InsertLineAbove) => self.insert_line_above(buffer),
            KeyAction::Simple(Action::InsertLine) => self.insert_line(),
            _ => (),
        }
    }

    fn insert_char(&mut self) {
        self.absolute_position += 1;
        self.col += 1;
    }

    fn insert_line(&mut self) {
        self.absolute_position += 1;
        self.col = 0;
        self.row += 1;
    }

    fn insert_line_below(&mut self, buffer: &mut TextObject) {
        self.col = 0;
        self.row += 1;
        if let Some(mark) = buffer.marker.get_by_line(self.row + 1) {
            self.absolute_position = mark.start;
        }
    }

    fn insert_line_above(&mut self, buffer: &mut TextObject) {
        self.col = 0;
        if let Some(mark) = buffer.marker.get_by_line(self.row + 1) {
            self.absolute_position = mark.start;
        }
    }

    fn delete_prev_char(&mut self, buffer: &mut TextObject) {
        match (self.col, self.row) {
            (0, 0) => (),
            (0, _) => {
                self.move_up(buffer);
                self.move_to_line_end(buffer);
            }
            _ => {
                self.col = self.col.saturating_sub(1);
                self.absolute_position = self.absolute_position.saturating_sub(1);
            }
        }
    }

    pub fn get_readable_position(&self) -> Position {
        Position {
            row: self.row + 1,
            col: self.col + 1,
        }
    }

    pub fn move_up(&mut self, buffer: &mut TextObject) {
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

    fn move_right(&mut self, buffer: &mut TextObject, mode: &Mode) {
        if let Some(mark) = buffer.marker.get_by_line(self.row + 1) {
            let limit = match mode {
                Mode::Normal => mark.size.saturating_sub(2),
                _ => mark.size.saturating_sub(1),
            };
            self.col = usize::min(self.col + 1, limit);
            self.absolute_position = mark.start + self.col;
        }
    }

    fn move_down(&mut self, buffer: &mut TextObject, mode: &Mode) {
        let next_line = self.row.add(2);
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

    fn move_left(&mut self, _: &mut TextObject) {
        if self.col > 0 {
            self.col -= 1;
            self.absolute_position -= 1;
        }
    }

    fn move_to_line_end(&mut self, buffer: &mut TextObject) {
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

    fn move_to_bottom(&mut self, buffer: &mut TextObject) {
        let total_lines = buffer.marker.len();
        let mark = buffer.marker.get_by_line(total_lines).unwrap();
        self.row = total_lines - 1;
        self.col = mark.size.saturating_sub(2);
        self.absolute_position = mark.start + mark.size.saturating_sub(2);
    }

    fn move_to_line_start(&mut self, buffer: &mut TextObject) {
        let mark = buffer.marker.get_by_line(self.row + 1).unwrap();
        self.col = 0;
        self.absolute_position = mark.start;
    }

    fn move_to_next_word(&mut self, buffer: &mut TextObject) {
        let content = buffer.to_string();
        let mut pos = self.absolute_position;

        let starting_char = content
            .chars()
            .nth(pos)
            .expect("current position should never be out of bounds");

        match starting_char {
            c if c.is_whitespace() => {
                while let Some(c) = content[pos..].chars().nth(0) {
                    if !c.is_whitespace() {
                        break;
                    }
                    pos += 1;
                }
            }
            c if c.is_alphanumeric() => {
                let mut found_whitespace = false;
                while let Some(c) = content[pos..].chars().nth(0) {
                    match (c, found_whitespace) {
                        (c, false) if c.is_whitespace() => {
                            found_whitespace = true;
                        }
                        (c, true) if c.is_alphanumeric() => {
                            break;
                        }
                        (c, _) if !c.is_whitespace() && !c.is_alphanumeric() => {
                            break;
                        }
                        _ => (),
                    }
                    pos += 1;
                }
            }
            _ => {
                let mut found_newline = false;
                while let Some(c) = content[pos..].chars().nth(0) {
                    match (c, found_newline) {
                        ('\n', _) => {
                            found_newline = true;
                        }
                        (c, _) if c.is_alphanumeric() => {
                            break;
                        }
                        (_, true) => {
                            let marker = buffer
                                .marker
                                .get_by_cursor(pos)
                                .expect("should never be out of bounds");
                            if marker.size == 1 || !c.is_whitespace() {
                                break;
                            }
                        }
                        _ => (),
                    }
                    pos += 1;
                }
            }
        }

        if let Some(mark) = buffer.marker.get_by_cursor(pos) {
            let offset = pos - mark.start;
            self.col = offset;
            self.row = mark.line - 1;
            self.absolute_position = pos;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cursor_move_up() {
        let gap = 5;
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Hello\nWorld\nEveryone", gap);
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
        let cursor = Cursor {
            row: 1,
            ..Default::default()
        };

        let pos = cursor.get_readable_position();

        assert_eq!(pos.col, 1);
        assert_eq!(pos.row, 2);
    }

    #[test]
    fn move_to_line_start() {
        let gap = 5;
        let mut cursor = Cursor::default();
        let mut buffer =
            TextObject::from_string(1, "Hello World! This is a big line\n this isn't\n", gap);

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
        let mut cursor = Cursor::default();
        let mut buffer =
            TextObject::from_string(1, "Hello World! This is a big line\n this isn't\n", gap);

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
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Hello\nWorld\nEveryone", gap);
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
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Hello\nWorld\nEveryone", gap);

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
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Hello\nWorld\nEveryone", gap);
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
        let mut cursor = Cursor::default();
        let mut buffer =
            TextObject::from_string(1, "Hello World! This is a big line\n this isn't\n", gap);

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
        let mut cursor = Cursor::default();
        let mut buffer =
            TextObject::from_string(1, "Hello\nWorld! This is a big line we got here", gap);
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
        let mut cursor = Cursor::default();
        let mut buffer =
            TextObject::from_string(1, "Hello World! This is a big line\nThis isn't", gap);
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
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Hello\nWorld! This is a big line", gap);
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
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Hello\nWorld!", gap);

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
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Hello World!", gap);
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
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Hello World!", gap);

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
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Hello World!", gap);
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

    #[test]
    fn test_move_to_top() {
        let gap = 5;
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Random\nmultiline\nstring\ntext\nbuffer", gap);

        for _ in 0..5 {
            cursor.handle(
                &KeyAction::Simple(Action::MoveDown),
                &mut buffer,
                &Mode::Normal,
            );
        }

        assert_eq!(cursor.row, 4);
        assert_eq!(cursor.col, 4);
        assert_eq!(cursor.absolute_position, 33);

        cursor.handle(
            &KeyAction::Simple(Action::MoveToTop),
            &mut buffer,
            &Mode::Normal,
        );

        assert_eq!(cursor.row, 0);
        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.absolute_position, 0);
    }

    #[test]
    fn test_move_to_bottom() {
        let gap = 5;
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Random\nmultiline\nstring\ntext\nbuffer", gap);

        cursor.handle(
            &KeyAction::Simple(Action::MoveToBottom),
            &mut buffer,
            &Mode::Normal,
        );

        assert_eq!(cursor.row, 4);
        assert_eq!(cursor.col, 4);
        assert_eq!(cursor.absolute_position, 33);
    }

    #[test]
    fn test_insert_char() {
        let gap = 5;
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Hello, World!", gap);

        cursor.handle(
            &KeyAction::Simple(Action::InsertChar('.')),
            &mut buffer,
            &Mode::Normal,
        );

        assert_eq!(cursor.row, 0);
        assert_eq!(cursor.col, 1);
        assert_eq!(cursor.absolute_position, 1);
    }

    #[test]
    fn test_delete_prev_char() {
        let gap = 5;
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Hello, World!", gap);
        cursor.col = 3;
        cursor.absolute_position = 3;

        cursor.handle(
            &KeyAction::Simple(Action::DeletePreviousChar),
            &mut buffer,
            &Mode::Normal,
        );

        assert_eq!(cursor.col, 2);
        assert_eq!(cursor.absolute_position, 2);
    }

    #[test]
    fn test_insert_line_below() {
        let gap = 5;
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Hello\nWorld!", gap);
        cursor.col = 3;
        cursor.absolute_position = 3;

        cursor.handle(
            &KeyAction::Simple(Action::InsertLineBelow),
            &mut buffer,
            &Mode::Normal,
        );

        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.absolute_position, 6);
        assert_eq!(cursor.row, 1);
    }

    #[test]
    fn test_insert_line_above() {
        let gap = 5;
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Hello\nWorld!", gap);
        cursor.col = 3;
        cursor.row = 1;
        cursor.absolute_position = 9;

        cursor.handle(
            &KeyAction::Simple(Action::InsertLineAbove),
            &mut buffer,
            &Mode::Normal,
        );

        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.absolute_position, 6);
        assert_eq!(cursor.row, 1);
    }

    #[test]
    fn test_insert_line() {
        let gap = 5;
        let mut cursor = Cursor::default();
        let mut buffer = TextObject::from_string(1, "Hello\nWorld!", gap);
        cursor.col = 3;
        cursor.row = 0;
        cursor.absolute_position = 6;

        cursor.handle(
            &KeyAction::Simple(Action::InsertLine),
            &mut buffer,
            &Mode::Normal,
        );

        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.absolute_position, 7);
        assert_eq!(cursor.row, 1);
    }
}
