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
}
