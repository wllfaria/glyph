use crate::theme::Style;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Cell {
    pub c: char,
    pub style: Style,
}
