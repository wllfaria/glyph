use crate::command::Command;

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

    pub fn handle(&self, command: Command) {}

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

    pub fn delete_char(&mut self, row: usize, col: usize) {}

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
