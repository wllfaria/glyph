use crate::pane::Position;

#[derive(Debug)]
pub struct Buffer {
    id: u16,
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn new(id: u16, filename: Option<String>) -> Self {
        let lines = match filename {
            Some(filename) => {
                let lines = std::fs::read_to_string(filename).unwrap();
                lines.lines().map(|s| s.to_string()).collect()
            }
            None => Vec::new(),
        };
        Buffer { id, lines }
    }

    pub fn new_line(&mut self, current_row: usize, col: usize) {
        match col {
            _ if self.lines.len() == 0 => {
                self.lines.push(String::new());
            }
            c if c < self.lines[current_row].len() => {
                self.split_line(current_row, col);
            }
            _ => {
                self.lines.insert(current_row + 1, String::new());
            }
        }
    }

    pub fn insert_char(&mut self, row: usize, col: usize, c: char) {
        if col >= self.lines[row].len() {
            self.lines[row].push(c);
            return;
        }
        self.lines[row].insert(col, c);
    }

    pub fn delete_char(&mut self, row: usize, col: usize) -> Position {
        match col {
            c if c == 0 && row == 0 => Position {
                col: 0,
                row: 0,
                render_col: 0,
            },
            c if c == 0 && row > 0 => {
                let cursor = Position {
                    col: self.get_line_len(row - 1) as u16,
                    row: row as u16 - 1,
                    render_col: self.get_line_len(row - 1) as u16,
                };
                self.append_line(row - 1);
                return cursor;
            }
            c if c >= self.lines[row].len() => {
                self.lines[row].pop();
                return Position {
                    col: self.get_line_len(row) as u16,
                    row: row as u16,
                    render_col: self.get_line_len(row) as u16,
                };
            }
            _ => {
                let left = self.lines[row][..col - 1].to_string();
                let right = self.lines[row][col..].to_string();
                self.lines[row] = left + &right;
                return Position {
                    col: col as u16 - 1,
                    row: row as u16,
                    render_col: col as u16 - 1,
                };
            }
        }
    }

    pub fn split_line(&mut self, line: usize, col: usize) {
        let first = self.lines[line][..col].to_string();
        let second = self.lines[line][col..].to_string();
        self.lines[line] = first.to_string();
        self.lines.insert(line + 1, String::new());
        self.lines[line + 1] = second.to_string();
    }

    pub fn append_line(&mut self, line: usize) {
        let next = self.lines[line + 1].to_string();
        self.lines[line].push_str(&next);
        self.lines.remove(line + 1);
    }

    pub fn get_line_len(&self, line: usize) -> usize {
        match line {
            l if l >= self.lines.len() => 0,
            _ if self.lines.len() == 0 => 0,
            _ => self.lines[line].len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::Buffer;

    fn make_buffer() -> Buffer {
        let buffer = Buffer::new(1, None);
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

        buffer.new_line(0, 0);
        buffer.new_line(0, 0);
        buffer.new_line(1, 0);
        buffer.insert_char(0, 0, 'a');
        buffer.insert_char(2, 10, 'b');
        buffer.new_line(0, 0);

        assert_eq!(buffer.lines.len(), 4);
        assert_eq!(buffer.lines[0], "");
        assert_eq!(buffer.lines[1], "a");
        assert_eq!(buffer.lines[2], "");
        assert_eq!(buffer.lines[3], "b");
    }

    #[test]
    fn test_insert_char() {
        let mut buffer = make_buffer();

        buffer.new_line(0, 0);
        buffer.new_line(0, 0);
        buffer.new_line(1, 0);
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

        buffer.new_line(0, 0);
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

        buffer.new_line(0, 0);
        let input = "Hello World!";

        for (i, ch) in input.chars().enumerate() {
            buffer.insert_char(0, i, ch);
        }
        buffer.new_line(0, 0);

        for (i, ch) in input.chars().enumerate() {
            buffer.insert_char(1, i, ch);
        }

        buffer.append_line(0);

        assert_eq!(buffer.lines.len(), 1);
        assert_eq!(buffer.lines[0], "Hello World!Hello World!");
    }
}
