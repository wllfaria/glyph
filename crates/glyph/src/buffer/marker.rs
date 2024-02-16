use super::vec_marker::VecMarker;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Mark {
    pub start: usize,
    pub line: usize,
    pub size: usize,
}

impl Mark {
    pub fn new(start: usize, line: usize, size: usize) -> Self {
        Self { start, line, size }
    }
}

pub trait Marker: std::fmt::Debug {
    fn add_mark(&mut self, mark: Mark, at: usize);
    fn del_mark(&mut self, at: usize);
    fn get_by_cursor(&self, position: usize) -> Option<Mark>;
    fn get_by_line(&self, line: usize) -> Option<Mark>;
    fn set_marks(&mut self, text: &[char]);
    fn len(&self) -> usize;
}

impl dyn Marker {
    pub fn get_marker() -> Box<Self> {
        Box::new(VecMarker::new())
    }
}
