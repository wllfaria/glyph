use crate::buffer::marker::{Mark, Marker};

use super::lines::Lines;

#[derive(Debug)]
pub struct VecMarker {
    marks: Vec<Mark>,
}

impl VecMarker {
    pub fn new() -> Self {
        Self { marks: Vec::new() }
    }

    fn update_marks(&mut self) {
        let mut total_size = 0;
        for (i, mark) in self.marks.iter_mut().enumerate() {
            mark.line = i + 1;
            mark.start = total_size;
            total_size += mark.size;
        }
    }
}

impl Marker for VecMarker {
    fn len(&self) -> usize {
        self.marks.len()
    }

    fn add_mark(&mut self, mark: Mark, at: usize) {
        self.marks.insert(at, mark);
        self.update_marks();
    }

    fn del_mark(&mut self, at: usize) {
        self.marks.remove(at);
        self.update_marks();
    }

    fn get_by_cursor(&self, position: usize) -> Option<Mark> {
        let index = self
            .marks
            .iter()
            .position(|m| position >= m.start && position < m.start + m.size);
        if let Some(index) = index {
            return Some(self.marks[index]);
        }
        None
    }

    fn get_by_line(&self, line: usize) -> Option<Mark> {
        let mark = self.marks.get(line.saturating_sub(1));
        if let Some(mark) = mark {
            return Some(*mark);
        }
        None
    }

    fn set_marks(&mut self, buffer: &[char]) {
        self.marks.clear();
        let lines = Lines {
            buffer,
            start: 0,
            end: buffer.len(),
        };
        let mut i = 1;
        for line in lines {
            let default = Mark::default();
            let prev = self.get_by_line(i).unwrap_or(default);
            let start = prev.start + prev.size;
            self.add_mark(Mark::new(start, i, line.len()), i - 1);
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::marker::Mark;

    #[test]
    fn test_add_mark() {
        let mut marker = VecMarker::new();
        marker.add_mark(Mark::new(0, 1, 10), 0);
        marker.add_mark(Mark::new(1, 2, 20), 1);
        marker.add_mark(Mark::new(0, 1, 30), 0);

        assert_eq!(marker.marks.len(), 3);
        assert_eq!(marker.marks[0], Mark::new(0, 1, 30));
        assert_eq!(marker.marks[1], Mark::new(30, 2, 10));
        assert_eq!(marker.marks[2], Mark::new(40, 3, 20));
    }

    #[test]
    fn test_del_mark() {
        let mut marker = VecMarker::new();
        marker.add_mark(Mark::new(0, 1, 10), 0);
        marker.add_mark(Mark::new(1, 2, 20), 1);
        marker.add_mark(Mark::new(0, 1, 30), 0);

        marker.del_mark(1);

        assert_eq!(marker.marks.len(), 2);
        assert_eq!(marker.marks[0], Mark::new(0, 1, 30));
        assert_eq!(marker.marks[1], Mark::new(30, 2, 20));
    }

    #[test]
    fn test_get_by_cursor() {
        let mut marker = VecMarker::new();
        marker.add_mark(Mark::new(0, 1, 10), 0);
        marker.add_mark(Mark::new(1, 2, 20), 1);
        marker.add_mark(Mark::new(0, 1, 30), 0);

        let mark = marker.get_by_cursor(36).unwrap();

        assert_eq!(mark, Mark::new(30, 2, 10));
    }

    #[test]
    fn test_get_by_line() {
        let mut marker = VecMarker::new();
        marker.add_mark(Mark::new(0, 1, 10), 0);
        marker.add_mark(Mark::new(1, 2, 20), 1);
        marker.add_mark(Mark::new(0, 1, 30), 0);

        let mark = marker.get_by_line(2).unwrap();

        assert_eq!(mark, Mark::new(30, 2, 10));
    }

    #[test]
    fn should_return_none_if_line_is_invalid() {
        let marker = VecMarker::new();

        let mark = marker.get_by_line(10);

        assert_eq!(mark, None);
    }

    #[test]
    fn should_return_none_if_cursor_is_invalid() {
        let marker = VecMarker::new();

        let mark = marker.get_by_cursor(10);

        assert_eq!(mark, None);
    }
}
