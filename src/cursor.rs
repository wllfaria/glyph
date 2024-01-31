#[derive(Debug)]
pub struct Cursor {
    pub row: u16,
    pub col: u16,
}

impl Cursor {
    pub fn new() -> Self {
        Self { row: 0, col: 0 }
    }
}
