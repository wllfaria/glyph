#[derive(Debug)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer { lines: Vec::new() }
    }
}
