use crate::cursor::Cursor;
use crate::document::DocumentId;
use crate::rect::Rect;

slotmap::new_key_type! {
    pub struct WindowId;
}

#[derive(Debug, Clone)]
pub struct Window {
    pub id: WindowId,
    pub document: DocumentId,
    cursor: Cursor,
    scroll: (usize, usize),
    pub area: Rect,
}

impl Window {
    pub fn new(document: DocumentId) -> Window {
        Window {
            id: WindowId::default(),
            document,
            cursor: Cursor::default(),
            scroll: (0, 0),
            area: Rect::default(),
        }
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    pub fn scroll(&self) -> (usize, usize) {
        self.scroll
    }
}
