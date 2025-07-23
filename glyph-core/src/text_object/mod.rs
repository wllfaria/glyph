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

    pub fn get_line(&self, line_idx: usize) -> Option<RopeSlice<'_>> {
        self.inner.get_line(line_idx)
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

    pub fn delete_whole_line(&mut self, line: usize) {
        let len_lines = self.len_lines();
        assert!(line < len_lines);

        let line_start = self.inner.line_to_char(line);
        let line_end = self.inner.line_to_char(line + 1);
        self.inner.remove(line_start..line_end);
    }
}