use std::ops::Sub;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn bottom(&self) -> u16 {
        self.y.saturating_add(self.height)
    }

    pub fn right(&self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub fn shrink_bottom(mut self, amount: u16) -> Self {
        self.height = self.height.saturating_sub(amount);
        self
    }
}

impl From<(u16, u16)> for Rect {
    fn from((width, height): (u16, u16)) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }
}

impl Sub for Rect {
    type Output = Rect;

    fn sub(self, rhs: Self) -> Self::Output {
        Rect {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}
