use std::io;

use crate::command::Command;

#[derive(Debug)]
pub struct Buffer {
    pub id: u16,
    pub buffer: Vec<char>,
    gap_start: usize,
    gap_end: usize,
    gap_size: usize,
}

impl Buffer {
    pub fn new(id: u16, filename: Option<String>) -> io::Result<Self> {
        let lines = match filename {
            Some(filename) => std::fs::read_to_string(filename)?,
            None => String::new(),
        };
        Ok(Buffer::from_string(id, &lines, 5))
    }

    pub fn from_string(id: u16, content: &str, gap: usize) -> Self {
        let mut buffer = vec!['\0'; gap];
        buffer.extend(content.chars());

        Buffer {
            id,
            buffer,
            gap_start: 0,
            gap_size: gap,
            gap_end: gap,
        }
    }

    pub fn insert_char(&mut self, char: char, cursor_pos: usize) {
        if self.gap_start == self.gap_end {
            self.resize_gap();
        }
        self.move_gap(cursor_pos);
        self.buffer[self.gap_start] = char;
        self.gap_start += 1;
    }

    fn resize_gap(&mut self) {
        let left = &self.buffer[0..self.gap_start + 1];
        let right = &self.buffer[self.gap_end..];
        let new_size = self.buffer.len() + self.gap_size;
        let mut new_buffer = vec!['\0'; new_size];
        new_buffer[0..left.len()].copy_from_slice(left);
        new_buffer[left.len() + self.gap_size - 1..left.len() + self.gap_size + right.len() - 1]
            .copy_from_slice(right);
        self.gap_end = new_buffer.len() - right.len();
        self.buffer = new_buffer;
    }

    pub fn delete_char(&mut self, cursor_pos: usize) {
        if cursor_pos == 0 {
            return;
        }
        self.move_gap(cursor_pos);
        self.gap_start -= 1;
    }

    pub fn move_gap(&mut self, cursor_pos: usize) {
        let cursor_pos = self.translate_cursor_pos(cursor_pos);

        if cursor_pos >= self.gap_end {
            for _ in self.gap_end..cursor_pos {
                self.buffer[self.gap_start] = self.buffer[self.gap_end];
                self.gap_start += 1;
                self.gap_end += 1;
            }
        } else {
            for _ in cursor_pos..self.gap_start {
                self.buffer[self.gap_end - 1] = self.buffer[self.gap_start - 1];
                self.gap_start -= 1;
                self.gap_end -= 1;
            }
        }
    }

    fn translate_cursor_pos(&self, cursor_pos: usize) -> usize {
        let left = &self.buffer[0..self.gap_start];
        match left.len().cmp(&cursor_pos) {
            std::cmp::Ordering::Greater => cursor_pos,
            std::cmp::Ordering::Equal => cursor_pos,
            std::cmp::Ordering::Less => {
                left.len() + (self.gap_end - self.gap_start) + (cursor_pos - left.len())
            }
        }
    }

    pub fn as_string(&self) -> String {
        let mut result = String::new();
        let left = &self.buffer[0..self.gap_start];
        let right = &self.buffer[self.gap_end..];
        result.push_str(&left.iter().collect::<String>());
        result.push_str(&right.iter().collect::<String>());
        result
    }

    pub fn handle(&self, _command: Command) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_initialization() {
        let gap = 5;
        let buffer = Buffer::from_string(1, "Hello, World!", gap);
        let needle = &"Hello, World!".chars().collect::<Vec<_>>();

        assert_eq!(buffer.gap_end - buffer.gap_start, gap);
        assert!(buffer.buffer[gap..].starts_with(needle));
    }

    #[test]
    fn test_move_gap() {
        let gap = 5;
        let mut buffer = Buffer::from_string(1, "Hello, World!", gap);
        let needle = &"Hello".chars().collect::<Vec<_>>();

        buffer.move_gap(5);

        assert_eq!(buffer.gap_start, 5);
        assert_eq!(buffer.gap_end, buffer.gap_start + gap);
        assert!(buffer.buffer[0..buffer.gap_start].starts_with(needle));
    }

    #[test]
    fn test_move_gap_left() {
        let gap = 5;
        let mut buffer = Buffer::from_string(1, "Hello, World!", gap);

        buffer.move_gap(13);

        assert_eq!(buffer.gap_start, 13);
        assert_eq!(buffer.gap_end, buffer.gap_start + gap);

        buffer.move_gap(5);

        assert_eq!(buffer.gap_start, 5);
        assert_eq!(buffer.gap_end, buffer.gap_start + gap);
    }

    #[test]
    fn test_insert_into_gap() {
        let mut buffer = Buffer::from_string(1, "Hello, World!", 5);
        let needle = &"Hello!".chars().collect::<Vec<_>>();

        buffer.insert_char('!', 5);

        assert_eq!(buffer.gap_start, 6);
        assert!(buffer.buffer[0..buffer.gap_start].starts_with(needle));
        assert_eq!(buffer.gap_end - buffer.gap_start, 4);
    }

    #[test]
    fn test_delete_from_gap() {
        let mut buffer = Buffer::from_string(1, "Hello, World!", 5);
        let needle = &"Hell".chars().collect::<Vec<_>>();

        buffer.delete_char(5);

        assert_eq!(buffer.gap_start, 4);
        assert!(buffer.buffer[0..buffer.gap_start].starts_with(needle));
        assert_eq!(buffer.gap_end - buffer.gap_start, 6);
    }

    #[test]
    fn test_delete_everything_to_the_left() {
        let mut buffer = Buffer::from_string(1, "Hello, World!", 5);
        let needle = &"".chars().collect::<Vec<_>>();

        // this moves the gap to the right by 5
        buffer.delete_char(5);

        // then this removes more chars than there are
        for _ in 0..100 {
            buffer.delete_char(buffer.gap_start);
        }

        assert_eq!(buffer.gap_start, 0);
        assert!(buffer.buffer[0..buffer.gap_start].starts_with(needle));
        assert_eq!(buffer.gap_end - buffer.gap_start, 10);
    }

    #[test]
    fn test_should_resize_gap() {
        let gap = 5;
        let mut buffer = Buffer::from_string(1, "Hello, World!", gap);
        let needle = "Hello_____, World!".chars().collect::<Vec<_>>();

        buffer.insert_char('_', 5);
        buffer.insert_char('_', buffer.gap_start);
        buffer.insert_char('_', buffer.gap_start);
        buffer.insert_char('_', buffer.gap_start);
        buffer.insert_char('_', buffer.gap_start);

        assert_eq!(buffer.gap_end - buffer.gap_end, 0);
        assert_eq!(buffer.gap_start, 10);
        assert_eq!(buffer.gap_end, 10);
        assert_eq!(buffer.buffer, needle);

        buffer.insert_char('!', buffer.gap_start);
        let needle = "Hello_____!".chars().collect::<Vec<_>>();

        assert_eq!(buffer.buffer[0..buffer.gap_start], needle);
        assert_eq!(buffer.gap_start, 11);
        assert_eq!(buffer.gap_end - buffer.gap_start, 4);
        assert_eq!(buffer.gap_end, 15);
        assert_eq!(buffer.buffer.len(), 23);
        assert_eq!(
            buffer.buffer[buffer.gap_end..],
            ", World!".chars().collect::<Vec<_>>()
        );
        assert_eq!(
            buffer.buffer[0..buffer.gap_start],
            "Hello_____!".chars().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_get_as_string() {
        let gap = 5;
        let buffer = Buffer::from_string(1, "Hello, World!", gap);
        let needle = "Hello, World!".to_string();

        let result = buffer.as_string();

        assert_eq!(result, needle);
    }

    #[test]
    fn test_get_as_string_edited() {
        let gap = 5;
        let mut buffer = Buffer::from_string(1, "Hello, World!", gap);
        let needle = "Hello, This is heavily edited, World!".to_string();
        let insert = "This is heavily edited, ".to_string();

        let start_from = 7;
        for (i, c) in insert.chars().enumerate() {
            buffer.insert_char(c, i + start_from);
        }

        let result = buffer.as_string();
        assert_eq!(result, needle);

        let insert = "lol! ".to_string();
        let needle = "Hello, lol! This is heavily edited, World!".to_string();
        for (i, c) in insert.chars().enumerate() {
            buffer.insert_char(c, i + start_from);
        }

        let result = buffer.as_string();
        assert_eq!(result, needle);
    }
}
