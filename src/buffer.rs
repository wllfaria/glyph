#[derive(Debug)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn new(filename: Option<String>) -> Self {
        if let Some(filename) = filename {
            let lines = std::fs::read_to_string(filename).unwrap();
            let lines = lines.lines().map(|s| s.to_string()).collect();
            Buffer { lines }
        } else {
            Buffer { lines: Vec::new() }
        }
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_buffer_new() {
        let buffer = super::Buffer::new(None);
        assert_eq!(buffer.lines.len(), 0);
    }

    #[test]
    fn test_new_line() {
        let mut buffer = super::Buffer::new(None);

        buffer.new_line(0);
        buffer.insert_char(0, 0, 'a');
        buffer.new_line(1);
        buffer.insert_char(1, 10, 'b');
        buffer.new_line(1);

        assert_eq!(buffer.lines.len(), 3);
        assert_eq!(buffer.lines[0], "a");
        assert_eq!(buffer.lines[1], "");
        assert_eq!(buffer.lines[2], "b");
    }

    #[test]
    fn test_insert_char() {
        let mut buffer = super::Buffer::new(None);

        buffer.new_line(0);
        buffer.insert_char(0, 0, 'a');
        buffer.new_line(1);
        buffer.insert_char(1, 0, 'b');
        buffer.new_line(2);

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
        let mut buffer = super::Buffer::new(None);

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
        let mut buffer = super::Buffer::new(None);

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
