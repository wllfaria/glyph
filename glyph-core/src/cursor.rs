use crate::document::Document;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cursor {
    x: usize,
    y: usize,
}

impl Cursor {
    pub fn new(x: usize, y: usize) -> Cursor {
        Cursor { x, y }
    }

    pub fn x(&self) -> usize {
        self.x
    }

    pub fn y(&self) -> usize {
        self.y
    }

    pub fn move_down(&mut self, document: &Document) {
        if let Some(_next_line) = document.text().get_line(self.y + 1) {
            self.y += 1;
        }
    }
}

impl std::fmt::Display for Cursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:{}", self.y + 1, self.x))
    }
}
