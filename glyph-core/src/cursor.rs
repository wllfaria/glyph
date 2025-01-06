use crate::document::Document;
use crate::editor::Mode;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cursor {
    x: usize,
    y: usize,
    /// when moving the cursor down or up, we will eventually move into smaller lines arbiratily, when
    /// that happens, virutal_x will hold the previous (larger) x value, so that we can restore the
    /// cursor column position when we move the cursor into a line that is larger than the virtual
    /// x, as a way of remembering where the cursor was when moving between lines
    virtual_x: usize,
}

impl Cursor {
    pub fn new(x: usize, y: usize) -> Cursor {
        Cursor { x, y, virtual_x: x }
    }

    pub fn x(&self) -> usize {
        self.x
    }

    pub fn y(&self) -> usize {
        self.y
    }

    pub fn move_to(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
        self.virtual_x = x;
    }

    pub fn move_left(&mut self) {
        self.x = self.x.saturating_sub(1);
        self.virtual_x = self.x;
    }

    pub fn move_down(&mut self, document: &Document, mode: Mode) {
        let Some(next_line) = document.text().get_line(self.y + 1) else {
            return;
        };

        self.y += 1;

        let next_line_len = next_line.len_chars();

        let end_offset = match mode {
            Mode::Normal => 2,
            Mode::Insert => 1,
            Mode::Command => 1,
            Mode::Visual => 1,
            Mode::VisualLine => 1,
            Mode::VisualBlock => 1,
        };
        if next_line_len <= self.virtual_x {
            self.x = if next_line_len == 0 { 0 } else { next_line_len.saturating_sub(end_offset) };
        } else {
            self.x = self.virtual_x;
        }
    }

    pub fn move_up(&mut self, document: &Document, mode: Mode) {
        if self.y == 0 {
            return;
        }

        self.y -= 1;
        let current_line = document.text().get_line(self.y).unwrap();
        let current_line_len = current_line.len_chars();

        let end_offset = match mode {
            Mode::Normal => 2,
            Mode::Insert => 1,
            Mode::Command => 1,
            Mode::Visual => 1,
            Mode::VisualLine => 1,
            Mode::VisualBlock => 1,
        };

        if current_line_len <= self.virtual_x {
            self.x = if current_line_len == 0 { 0 } else { current_line_len.saturating_sub(end_offset) };
        } else {
            self.x = self.virtual_x;
        }
    }

    pub fn move_right(&mut self, document: &Document, mode: Mode) {
        let end_offset = match mode {
            Mode::Normal => 2,
            Mode::Insert => 1,
            Mode::Command => 1,
            Mode::Visual => 1,
            Mode::VisualLine => 1,
            Mode::VisualBlock => 1,
        };
        if let Some(line) = document.text().get_line(self.y) {
            self.x = (line.len_chars().saturating_sub(end_offset)).min(self.x + 1);
            self.virtual_x = self.x;
        }
    }
}

impl std::fmt::Display for Cursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:{}", self.y + 1, self.x))
    }
}
