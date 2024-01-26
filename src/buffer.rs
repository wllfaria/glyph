use std::{cell::RefCell, io::Result, rc::Rc};

use crate::{commands::Directions, pane::Pane, state::State};

#[derive(Debug)]
pub struct Buffer {
    id: u16,
    pub lines: Vec<String>,
    panes: Vec<Rc<RefCell<Pane>>>,
    state: Rc<RefCell<State>>,
}

impl Buffer {
    pub fn new(id: u16, filename: Option<String>, state: Rc<RefCell<State>>) -> Self {
        let lines = match filename {
            Some(filename) => {
                let lines = std::fs::read_to_string(filename).unwrap();
                lines.lines().map(|s| s.to_string()).collect()
            }
            None => Vec::new(),
        };
        Buffer {
            id,
            lines,
            panes: Vec::new(),
            state,
        }
    }

    fn notify_panes(&self) -> Result<()> {
        for pane in &self.panes {
            pane.borrow_mut().render()?;
        }
        Ok(())
    }

    pub fn new_line(&mut self, at: usize) {
        self.lines.insert(at, String::new());
    }

    pub fn insert_char(&mut self, line: usize, col: usize, c: char) {
        if col >= self.lines[line].len() {
            self.lines[line].push(c);
            return;
        }
        self.lines[line].insert(col, c);
    }

    pub fn delete_char(&mut self, line: usize, col: usize) {
        if col >= self.lines[line].len() {
            self.lines[line].pop();
            return;
        }
        let left = self.lines[line][..col].to_string();
        let right = self.lines[line][col + 1..].to_string();
        self.lines[line] = left + &right;
    }

    pub fn split_line(&mut self, line: usize, col: usize) {
        let first = self.lines[line][..col].to_string();
        let second = self.lines[line][col..].to_string();
        self.lines[line] = first.to_string();
        self.new_line(line + 1);
        self.lines[line + 1] = second.to_string();
    }

    pub fn append_line(&mut self, line: usize) {
        let next = self.lines[line + 1].to_string();
        self.lines[line].push_str(&next);
        self.lines.remove(line + 1);
    }

    pub fn get_line_len(&self, line: usize) -> usize {
        if line >= self.lines.len() {
            return 0;
        }
        self.lines[line].len()
    }
}

#[cfg(test)]
mod tests {
    use crate::{buffer::Buffer, state::State};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn make_buffer() -> Buffer {
        let state = Rc::new(RefCell::new(State::new()));
        let buffer = Buffer::new(1, None, Rc::clone(&state));
        buffer
    }

    #[test]
    fn test_buffer_new() {
        let buffer = make_buffer();
        assert_eq!(buffer.lines.len(), 0);
    }

    #[test]
    fn test_new_line() {
        let mut buffer = make_buffer();

        buffer.new_line(0);
        buffer.new_line(1);
        buffer.new_line(2);
        buffer.insert_char(0, 0, 'a');
        buffer.insert_char(2, 10, 'b');
        buffer.new_line(0);

        assert_eq!(buffer.lines.len(), 4);
        assert_eq!(buffer.lines[0], "");
        assert_eq!(buffer.lines[1], "a");
        assert_eq!(buffer.lines[2], "");
        assert_eq!(buffer.lines[3], "b");
    }

    #[test]
    fn test_insert_char() {
        let mut buffer = make_buffer();

        buffer.new_line(0);
        buffer.new_line(1);
        buffer.new_line(2);
        buffer.insert_char(0, 0, 'a');
        buffer.insert_char(1, 0, 'b');

        let input = "Hello World!";

        for (i, ch) in input.chars().enumerate() {
            buffer.insert_char(2, i, ch);
        }

        assert_eq!(buffer.lines.len(), 3);
        assert_eq!(buffer.lines[0], "a");
        assert_eq!(buffer.lines[1], "b");
        assert_eq!(buffer.lines[2], "Hello World!");

        buffer.insert_char(2, 5, ',');

        assert_eq!(buffer.lines[2], "Hello, World!");
    }

    #[test]
    fn test_split_line() {
        let mut buffer = make_buffer();

        buffer.new_line(0);
        let input = "Hello World!";

        for (i, ch) in input.chars().enumerate() {
            buffer.insert_char(0, i, ch);
        }

        buffer.split_line(0, 5);

        assert_eq!(buffer.lines.len(), 2);
        assert_eq!(buffer.lines[0], "Hello");
        assert_eq!(buffer.lines[1], " World!");
    }

    #[test]
    fn test_append_line() {
        let mut buffer = make_buffer();

        buffer.new_line(0);
        let input = "Hello World!";

        for (i, ch) in input.chars().enumerate() {
            buffer.insert_char(0, i, ch);
        }
        buffer.new_line(1);

        for (i, ch) in input.chars().enumerate() {
            buffer.insert_char(1, i, ch);
        }

        buffer.append_line(0);

        assert_eq!(buffer.lines.len(), 1);
        assert_eq!(buffer.lines[0], "Hello World!Hello World!");
    }
}
