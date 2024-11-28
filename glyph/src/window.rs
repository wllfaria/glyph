use glyph_term::graphics::Rect;

use crate::cursor::Cursor;
use crate::document::DocumentId;

slotmap::new_key_type! {
    pub struct WindowId;
}

#[derive(Debug, Clone)]
pub struct Window {
    pub id: WindowId,
    pub document: DocumentId,
    pub cursor: Cursor,
    pub area: Rect,
}

impl Window {
    pub fn new(document: DocumentId) -> Window {
        Window {
            id: WindowId::default(),
            document,
            cursor: Cursor::default(),
            area: Rect::default(),
        }
    }
}
