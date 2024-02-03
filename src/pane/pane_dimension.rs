#[derive(Debug)]
pub struct PaneDimensions {
    pub row: u16,
    pub col: u16,
    pub height: u16,
    pub width: u16,
}

impl From<(u16, u16)> for PaneDimensions {
    fn from((width, height): (u16, u16)) -> Self {
        Self {
            col: 0,
            row: 0,
            width,
            height,
        }
    }
}
