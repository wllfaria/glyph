use theme::Style;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Cell {
    pub c: char,
    pub style: Style,
}

impl Cell {
    pub fn new(c: char, style: Style) -> Self {
        Self { c, style }
    }
}
