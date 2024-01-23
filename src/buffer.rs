#[derive(Debug)]
pub struct CursorPosition {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug)]
pub struct Buffer {
    pub cursor_position: CursorPosition,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            cursor_position: CursorPosition { x: 0, y: 0 },
        }
    }
}
