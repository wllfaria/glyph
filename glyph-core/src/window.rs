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
}
