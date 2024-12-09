use crate::document::DocumentId;
use crate::rect::Rect;

slotmap::new_key_type! {
    pub struct WindowId;
}

#[derive(Debug, Clone)]
pub struct Window {
    pub id: WindowId,
    pub document: DocumentId,
    scroll: (usize, usize),
    pub area: Rect,
}

impl Window {
    pub fn new(document: DocumentId) -> Window {
        Window {
            id: WindowId::default(),
            document,
            scroll: (0, 0),
            area: Rect::default(),
        }
    }

    pub fn scroll(&self) -> (usize, usize) {
        self.scroll
    }

    pub fn scroll_left(&mut self) {
        self.scroll.0 = self.scroll.0.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll.1 += 1;
    }

    pub fn scroll_up(&mut self) {
        self.scroll.1 = self.scroll.1.saturating_sub(1);
    }

    pub fn scroll_right(&mut self) {
        self.scroll.0 += 1;
    }
}
