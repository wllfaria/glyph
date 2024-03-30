use super::cell::Cell;

#[derive(Debug)]
pub struct Change<'a> {
    pub cell: &'a Cell,
    pub row: u16,
    pub col: u16,
}
