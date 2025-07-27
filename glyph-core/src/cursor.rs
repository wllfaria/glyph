use crate::buffer_manager::Buffer;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub virtual_x: usize,
}

impl Cursor {
    pub fn move_left_by(&mut self, amount: usize) {
        self.x = self.x.saturating_sub(amount);
        self.virtual_x = self.x;
    }

    pub fn move_to(&mut self, buffer: &Buffer, x: usize, y: usize) {
        let content = buffer.content();
        let total_lines = content.len_lines();
        assert!(y < total_lines);

        let line_len = content.line_len(y);
        assert!(x < line_len);

        self.x = x;
        self.y = y;
        self.virtual_x = self.x;
    }

    pub fn move_to_with_offset(&mut self, buffer: &Buffer, x: usize, y: usize, offset: usize) {
        let content = buffer.content();
        let total_lines = content.len_lines();
        assert!(y < total_lines);

        let line_len = content.line_len(y).saturating_sub(offset);
        self.x = usize::min(x, line_len);
        self.y = y;
        self.virtual_x = self.x;
    }

    pub fn move_down_by(&mut self, buffer: &Buffer, amount: usize) {
        let content = buffer.content();
        let total_lines = content.len_lines();
        let max_y = total_lines.saturating_sub(1);
        self.y = usize::min(self.y.saturating_add(amount), max_y);
    }

    pub fn move_up_by(&mut self, amount: usize) {
        self.y = self.y.saturating_sub(amount);
    }

    pub fn move_right_by_with_offset(&mut self, buffer: &Buffer, amount: usize, offset: usize) {
        let content = buffer.content();
        let line_len = content.line_len(self.y);
        let max_x = line_len.saturating_sub(offset);
        self.x = usize::min(self.x.saturating_add(amount), max_x);
        self.virtual_x = self.x;
    }

    pub fn move_to_line_start(&mut self) {
        self.x = 0;
        self.virtual_x = 0;
    }

    pub fn move_to_line_end_with_offset(&mut self, buffer: &Buffer, offset: usize) {
        self.move_right_by_with_offset(buffer, usize::MAX, offset);
        self.virtual_x = usize::MAX;
    }
}