/// A structure that represents a size.
///
/// The values depend on the renderer. For a TUI renderer, this would be the
/// number of columns and rows. For a GUI renderer, this would be the width and
/// height of the window, most likely in pixels.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

impl From<(u16, u16)> for Size {
    fn from((width, height): (u16, u16)) -> Self {
        Self::new(width, height)
    }
}

impl From<(u8, u8)> for Size {
    fn from((width, height): (u8, u8)) -> Self {
        Self::new(width as u16, height as u16)
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn with_size(x: u16, y: u16, size: Size) -> Self {
        Self {
            x,
            y,
            width: size.width,
            height: size.height,
        }
    }

    pub fn cut_left(&mut self, amount: u16) {
        assert!(amount <= self.width);
        self.x += amount;
        self.width -= amount;
    }

    pub fn cut_bottom(&mut self, amount: u16) {
        assert!(amount <= self.height);
        self.height -= amount;
    }

    pub fn cut_top(&mut self, amount: u16) {
        assert!(amount <= self.height);
        self.y += amount;
        self.height -= amount;
    }

    pub fn cut_right(&mut self, amount: u16) {
        assert!(amount <= self.width);
        self.width -= amount;
    }
}

impl From<Size> for Rect {
    fn from(size: Size) -> Self {
        Self {
            x: 0,
            y: 0,
            width: size.width,
            height: size.height,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Point<T: Default> {
    pub x: T,
    pub y: T,
}

impl<T: Default> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Default> From<(T, T)> for Point<T> {
    fn from((x, y): (T, T)) -> Self {
        Self::new(x, y)
    }
}
