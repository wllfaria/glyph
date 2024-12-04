use crate::color::Color;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct HighlightGroup {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
}
