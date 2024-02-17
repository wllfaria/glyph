mod lines;
pub mod marker;
mod vec_marker;

use std::io;

use crate::buffer::lines::Lines;
use crate::buffer::marker::Marker;
use crate::command::{BufferCommands, Command};
use marker::Mark;

#[derive(Debug)]
pub struct Buffer {
    pub id: u16,
    pub buffer: Vec<char>,
    pub marker: Box<dyn Marker>,
    pub file_name: String,
    gap_start: usize,
    gap_end: usize,
    gap_size: usize,
}

impl Buffer {
    pub fn new(id: u16, file_name: Option<String>) -> io::Result<Self> {
        let lines = match file_name {
            Some(ref name) => std::fs::read_to_string(name)?,
            None => String::new(),
        };
        let gap = 1000;
        let mut buffer = Buffer::from_string(id, &lines, gap);
        buffer.file_name = file_name.unwrap_or_default();
        Ok(buffer)
    }

    pub fn from_string(id: u16, content: &str, gap: usize) -> Self {
        let mut buffer = vec!['\0'; gap];
        buffer.extend(content.chars());
        let mut marker = <dyn Marker>::get_marker();
        marker.set_marks(&buffer);

        Buffer {
            id,
            buffer,
            gap_start: 0,
            gap_size: gap,
            gap_end: gap,
            marker,
            file_name: String::new(),
        }
    }

    pub fn insert_char(&mut self, char: char, cursor_pos: usize) {
        logger::debug!("Inserting char at {}", cursor_pos);
        self.move_gap(cursor_pos);
        self.buffer[self.gap_start] = char;
        self.gap_start += 1;
        if self.gap_start == self.gap_end {
            self.resize_gap();
        }
        self.marker.set_marks(&self.buffer);
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
        logger::error!("cursor {}", cursor_pos);
        if cursor_pos == 0 {
            return;
        }
        self.move_gap(cursor_pos);
        self.gap_start -= 1;
        self.buffer[self.gap_start] = '\0';
        self.marker.set_marks(&self.buffer);
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

    pub fn content_from(&self, line: usize, height: usize) -> String {
        self.to_string()
            .lines()
            .skip(line)
            .take(height)
            .collect::<Vec<&str>>()
            .join("\n")
    }

    pub fn line_from_mark(&self, mark: &Mark) -> String {
        let pos = self.translate_cursor_pos(mark.start);
        let mut lines = Lines {
            buffer: &self.buffer,
            start: pos,
            end: self.buffer.len(),
        };
        match lines.next() {
            Some(l) => l.iter().collect::<String>(),
            _ => String::new(),
        }
    }

    fn try_save(&self) -> std::io::Result<()> {
        if let Ok(mut path) = std::env::current_dir() {
            path.push(&self.file_name);
            logger::debug!("saving file: {:?}", path);
            std::fs::write(path, self.to_string())?;
        }
        Ok(())
    }

    pub fn handle(&mut self, command: &Command, cursor_pos: usize) -> std::io::Result<()> {
        match command {
            Command::Buffer(BufferCommands::Type(c)) => self.insert_char(*c, cursor_pos),
            Command::Buffer(BufferCommands::Backspace) => self.delete_char(cursor_pos),
            Command::Buffer(BufferCommands::NewLine) => self.insert_char('\n', cursor_pos),
            Command::Buffer(BufferCommands::Save) => self.try_save()?,
            _ => (),
        };
        Ok(())
    }
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
    use crate::command::*;

    use super::*;
    use crate::buffer::marker::Mark;

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
    fn test_marks_initialization() {
        let gap = 5;
        let buffer = Buffer::from_string(1, "Hello, World!", gap);

        assert_eq!(buffer.marker.len(), 1);
        assert_eq!(buffer.marker.get_by_line(0).unwrap(), Mark::new(0, 1, 13));
    }

    #[test]
    fn test_return_line_from_mark() {
        let gap = 5;
        let buffer = Buffer::from_string(1, "Hello, World!", gap);
        let mark = Mark {
            size: 13,
            line: 1,
            start: 0,
        };

        let line = buffer.line_from_mark(&mark);
        assert_eq!(line, "Hello, World!");
    }

    #[test]
    fn test_return_empty_line_from_invalid_mark() {
        let gap = 5;
        let buffer = Buffer::from_string(1, "Hello, World!", gap);
        let mark = Mark {
            size: 10,
            line: 2,
            start: 14 + gap,
        };

        let line = buffer.line_from_mark(&mark);
        assert_eq!(line, "");
    }

    #[test]
    fn test_insert_char_through_command() {
        let mut buffer = Buffer::from_string(1, "Hello, World!", 5);
        let first_needle = &"Hello!".chars().collect::<Vec<_>>();

        _ = buffer.handle(&Command::Buffer(BufferCommands::Type('!')), 5);

        assert_eq!(buffer.gap_start, 6);
        assert!(buffer.buffer[0..buffer.gap_start].starts_with(first_needle));
        assert_eq!(buffer.gap_end - buffer.gap_start, 4);
    }

    #[test]
    fn test_initialization_with_empty_filename() {
        let gap = 1000;
        let buffer = Buffer::new(1, None).unwrap();

        assert_eq!(buffer.buffer.len(), gap);
        assert_eq!(buffer.gap_start, 0);
        assert_eq!(buffer.gap_end - buffer.gap_start, gap);
        assert_eq!(buffer.gap_end, gap);
    }

    #[test]
    fn test_errors_with_invalid_filename() {
        let buffer = Buffer::new(1, Some(String::from("some_invalid_filename.extension")));

        assert!(buffer.is_err());
    }

    #[test]
    fn test_delete_char_through_command() {
        let mut buffer = Buffer::from_string(1, "Hello, World!", 5);
        let first_needle = &"Hell".chars().collect::<Vec<_>>();

        let _ = buffer.handle(&Command::Buffer(BufferCommands::Backspace), 5);

        assert_eq!(buffer.gap_start, 4);
        assert!(buffer.buffer[0..buffer.gap_start].starts_with(first_needle));
        assert_eq!(buffer.gap_end - buffer.gap_start, 6);
    }

    #[test]
    fn test_insert_newline_through_command() {
        let mut buffer = Buffer::from_string(1, "Hello, World!", 5);
        let first_needle = &"Hello".chars().collect::<Vec<_>>();
        let second_needle = &", World!".chars().collect::<Vec<_>>();

        let _ = buffer.handle(&Command::Buffer(BufferCommands::NewLine), 5);

        assert_eq!(buffer.gap_start, 6);
        assert!(buffer.buffer[0..buffer.gap_start].starts_with(first_needle));
        assert!(buffer.buffer[buffer.gap_end..].starts_with(second_needle));
        assert_eq!(buffer.gap_end - buffer.gap_start, 4);
    }
}
