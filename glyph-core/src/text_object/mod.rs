use ropey::iter::Lines;
use ropey::{Rope, RopeSlice};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextObject {
    inner: Rope,
}

impl TextObject {
    pub fn new(content: String) -> Self {
        Self {
            inner: Rope::from(content),
        }
    }

    pub fn line(&self, line_idx: usize) -> RopeSlice<'_> {
        assert!(line_idx < self.len_lines());
        self.inner.line(line_idx)
    }

    pub fn line_len(&self, line_idx: usize) -> usize {
        self.line(line_idx).len_chars()
    }

    pub fn len_lines(&self) -> usize {
        self.inner.len_lines()
    }

    pub fn lines(&self) -> Lines<'_> {
        self.inner.lines()
    }
}