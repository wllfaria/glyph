use crate::command::Command;

#[derive(Debug)]
pub struct Buffer {
    pub id: u16,
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

    pub fn handle(&self, _command: Command) {}
}
