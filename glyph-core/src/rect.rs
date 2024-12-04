use crate::cursor::Cursor;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl From<Cursor> for Point {
    fn from(value: Cursor) -> Point {
        Point {
            x: value.x() as u16,
            y: value.y() as u16,
        }
    }
}

impl From<&Cursor> for Point {
    fn from(value: &Cursor) -> Point {
        Point {
            x: value.x() as u16,
            y: value.y() as u16,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Rect {
        Rect { x, y, width, height }
    }

    /// creates a new Rect within bounds of a previous Rect, shrinking the size of
    /// the former Rect from the left
    ///
    /// # Panics
    /// panics if the original rect's height doesn't fit the new rect
    pub fn split_left(&mut self, size: u16) -> Rect {
        if self.width <= size {
            panic!("new rect doesn't fit inside the original rect");
        }

        let rect = Rect::new(self.x, self.y, size, self.height);
        self.width -= size;
        self.x += size;
        rect
    }

    /// creates a new Rect within bounds of a previous Rect, shrinking the size of
    /// the former Rect from the bottom
    ///
    /// # Panics
    /// panics if the original rect's height doesn't fit the new rect
    pub fn split_bottom(&mut self, size: u16) -> Rect {
        if self.height <= size {
            panic!("new rect doesn't fit inside the original rect");
        }

        let rect = Rect::new(self.x, self.height - size, self.width, size);
        self.height -= size;
        rect
    }

    /// creates a new Rect within bounds of a previous Rect, shrinking the size of
    /// the former Rect from the top
    ///
    /// # Panics
    /// panics if the original rect's height doesn't fit the new rect
    pub fn split_top(&mut self, size: u16) -> Rect {
        if self.height <= size {
            panic!("new rect doesn't fit inside the original rect");
        }

        let rect = Rect::new(self.x, self.y, self.width, size);
        self.height -= size;
        self.y += size;
        rect
    }

    /// creates a new Rect within bounds of a previous Rect, shrinking the size of
    /// the former Rect from the right
    ///
    /// # Panics
    /// panics if the original rect's height doesn't fit the new rect
    pub fn split_right(&mut self, size: u16) -> Rect {
        if self.width <= size {
            panic!("new rect doesn't fit inside the original rect");
        }

        let rect = Rect::new(self.width - size, self.y, size, self.height);
        self.width -= size;
        rect
    }
}