use std::num::NonZeroUsize;

use crate::document::DocumentId;
use crate::rect::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WindowId(NonZeroUsize);

impl WindowId {
    pub fn new(window: usize) -> Option<WindowId> {
        Some(WindowId(NonZeroUsize::new(window)?))
    }
}

impl Default for WindowId {
    fn default() -> Self {
        // Safety: 1 is non-zero
        WindowId(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}

impl WindowId {
    pub fn next(&self) -> WindowId {
        // Safety: will always be non-zero and less than usize::max + 1
        WindowId(unsafe { NonZeroUsize::new_unchecked(self.0.get().saturating_add(1)) })
    }
}

impl From<WindowId> for usize {
    fn from(value: WindowId) -> Self {
        value.0.into()
    }
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
