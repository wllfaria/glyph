use std::io;

use crate::buffer::lines::Lines;
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
        let gap = 1000;
        Ok(Buffer::from_string(id, &lines, gap))
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
        self.move_gap(cursor_pos);
        self.buffer[self.gap_start] = char;
        self.gap_start += 1;
        if self.gap_start == self.gap_end {
            self.resize_gap();
        }
    }

    fn resize_gap(&mut self) {
        let left = &self.buffer[0..self.gap_start];
        let right = &self.buffer[self.gap_end..];
        let new_size = self.buffer.len() + self.gap_size;
        let mut new_buffer = vec!['\0'; new_size];
        new_buffer[0..left.len()].copy_from_slice(left);
        new_buffer[left.len() + self.gap_size..left.len() + self.gap_size + right.len()]
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
        self.buffer[self.gap_start] = '\0';
    }

    pub fn move_gap(&mut self, cursor_pos: usize) {
        let cursor_pos = self.translate_cursor_pos(cursor_pos);

        if cursor_pos >= self.gap_end {
            for _ in self.gap_end..cursor_pos {
                self.buffer[self.gap_start] = self.buffer[self.gap_end];
                self.buffer[self.gap_end] = '\0';
                self.gap_start += 1;
                self.gap_end += 1;
            }
        } else {
            println!("{:?}", self.buffer);
            for _ in (cursor_pos..self.gap_start).rev() {
                self.gap_end -= 1;
                self.gap_start -= 1;
                self.buffer[self.gap_end] = self.buffer[self.gap_start];
                self.buffer[self.gap_start] = '\0';
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

    pub fn lines(&self) -> Lines {
        Lines {
            buffer: &self.buffer,
            start: 0,
            end: self.buffer.len(),
        }
    }

    pub fn lines_from(&self, start: usize) -> Lines {
        Lines {
            buffer: &self.buffer,
            start,
            end: self.buffer.len(),
        }
    }

    pub fn handle(&self, _command: Command) {}
}

impl std::fmt::Display for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lines = self
            .buffer
            .iter()
            .filter(|c| **c != '\0')
            .collect::<String>();
        write!(f, "{}", lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_initialization() {
        let gap = 5;
        let buffer = Buffer::from_string(1, "Hello, World!", gap);
        let first_needle = &"Hello, World!".chars().collect::<Vec<_>>();

        assert_eq!(buffer.gap_end - buffer.gap_start, gap);
        assert!(buffer.buffer[gap..].starts_with(first_needle));
    }

    #[test]
    fn test_move_gap() {
        let gap = 5;
        let mut buffer = Buffer::from_string(1, "Hello, World!", gap);
        let first_needle = &"Hello".chars().collect::<Vec<_>>();

        buffer.move_gap(5);

        assert_eq!(buffer.gap_start, 5);
        assert_eq!(buffer.gap_end, buffer.gap_start + gap);
        assert!(buffer.buffer[0..buffer.gap_start].starts_with(first_needle));
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
        let first_needle = &"Hello!".chars().collect::<Vec<_>>();

        buffer.insert_char('!', 5);

        assert_eq!(buffer.gap_start, 6);
        assert!(buffer.buffer[0..buffer.gap_start].starts_with(first_needle));
        assert_eq!(buffer.gap_end - buffer.gap_start, 4);
    }

    #[test]
    fn test_delete_from_gap() {
        let mut buffer = Buffer::from_string(1, "Hello, World!", 5);
        let first_needle = &"Hell".chars().collect::<Vec<_>>();
        let insert = "\nanother string\n";

        for (i, c) in insert.chars().enumerate() {
            buffer.insert_char(c, i + 5);
        }

        assert_eq!(buffer.to_string(), "Hello\nanother string\n, World!");

        buffer.delete_char(5);

        assert_eq!(buffer.to_string(), "Hell\nanother string\n, World!");
        assert_eq!(buffer.gap_start, 4);
        assert!(buffer.buffer[0..buffer.gap_start].starts_with(first_needle));
        assert_eq!(buffer.gap_end - buffer.gap_start, 5);
    }

    #[test]
    fn test_delete_everything_to_the_left() {
        let mut buffer = Buffer::from_string(1, "Hello, World!", 5);
        let first_needle = &"".chars().collect::<Vec<_>>();

        // this moves the gap to the right by 5
        buffer.delete_char(5);

        // then this removes more chars than there are
        for _ in 0..100 {
            buffer.delete_char(buffer.gap_start);
        }

        assert_eq!(buffer.gap_start, 0);
        assert!(buffer.buffer[0..buffer.gap_start].starts_with(first_needle));
        assert_eq!(buffer.gap_end - buffer.gap_start, 10);
    }

    #[test]
    fn test_should_resize_gap() {
        let gap = 5;
        let mut buffer = Buffer::from_string(1, "Hello, World!", gap);

        buffer.insert_char('_', 5);
        buffer.insert_char('_', buffer.gap_start);
        buffer.insert_char('_', buffer.gap_start);
        buffer.insert_char('_', buffer.gap_start);
        buffer.insert_char('_', buffer.gap_start);

        assert_eq!(buffer.gap_end - buffer.gap_end, 0);
        assert_eq!(buffer.gap_start, 10);
        assert_eq!(buffer.gap_end, 15);
        assert_eq!(buffer.buffer.len(), 23);

        buffer.insert_char('!', buffer.gap_start);
        let first_needle = "Hello_____!".chars().collect::<Vec<_>>();

        assert_eq!(buffer.buffer[0..buffer.gap_start], first_needle);
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
    fn test_get_lines() {
        let gap = 5;
        let multiline = r#"Hello, World!
This is a multiline string"#;
        let buffer = Buffer::from_string(1, multiline, gap);
        let first_needle = [
            'H', 'e', 'l', 'l', 'o', ',', ' ', 'W', 'o', 'r', 'l', 'd', '!', '\n',
        ];

        let second_needle = [
            'T', 'h', 'i', 's', ' ', 'i', 's', ' ', 'a', ' ', 'm', 'u', 'l', 't', 'i', 'l', 'i',
            'n', 'e', ' ', 's', 't', 'r', 'i', 'n', 'g',
        ];

        let mut lines = buffer.lines();
        let first = lines.next().unwrap();
        let second = lines.next().unwrap();

        assert_eq!(first, first_needle);
        assert_eq!(second, second_needle);
    }

    #[test]
    fn test_get_lines_edited() {
        let gap = 5;
        let mut buffer = Buffer::from_string(1, "Hello, World!", gap);
        let first_needle = [
            'H', 'e', 'l', 'l', 'o', ',', ' ', 'T', 'h', 'i', 's', ' ', 'i', 's', ' ', 'h', 'e',
            'a', 'v', 'i', 'l', 'y', ' ', 'e', 'd', 'i', 't', 'e', 'd', ',', ' ', '\n',
        ];
        let insert = "This is heavily edited, \n".to_string();

        let start_from = 7;
        for (i, c) in insert.chars().enumerate() {
            buffer.insert_char(c, i + start_from);
        }

        println!("{:?}", buffer.to_string());

        let mut lines = buffer.lines();
        let first = lines.next().unwrap();
        assert_eq!(first, first_needle);

        let insert = "lol! \n".to_string();
        for (i, c) in insert.chars().enumerate() {
            buffer.insert_char(c, i + start_from);
        }

        println!("{:?}", buffer.to_string());
        let mut lines = buffer.lines();
        let first = lines.next().unwrap();
        let second = lines.next().unwrap();
        let third = lines.next().unwrap();
        let fourth = lines.next();
        let first_needle = [
            'H', 'e', 'l', 'l', 'o', ',', ' ', 'l', 'o', 'l', '!', ' ', '\n',
        ];
        let second_needle = [
            'T', 'h', 'i', 's', ' ', 'i', 's', ' ', 'h', 'e', 'a', 'v', 'i', 'l', 'y', ' ', 'e',
            'd', 'i', 't', 'e', 'd', ',', ' ', '\n',
        ];
        let third_needle = ['W', 'o', 'r', 'l', 'd', '!'];
        assert_eq!(first, first_needle);
        assert_eq!(second, second_needle);
        assert_eq!(third, third_needle);
        assert_eq!(fourth, None);

        buffer.delete_char(3);

        println!("{:?}", buffer.to_string());

        let mut lines = buffer.lines();
        let first = lines.next().unwrap();
        let first_needle = ['H', 'e', 'l', 'o', ',', ' ', 'l', 'o', 'l', '!', ' ', '\n'];

        assert_eq!(first, first_needle);
    }
}
