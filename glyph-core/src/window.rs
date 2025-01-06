use std::num::NonZeroUsize;
use std::ops::Range;

use crate::document::DocumentId;
use crate::rect::{self, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Selection {
    pub start: Point,
    pub end: Point,
}

impl Selection {
    pub fn to_range(self, scroll: Point, area: Rect) -> Range<rect::Point> {
        let screen_start = Point {
            x: self.start.x.saturating_sub(scroll.x).max(area.x.into()),
            y: self.start.y.saturating_sub(scroll.y).max(area.y.into()),
        };

        let screen_end = Point {
            x: self.end.x.saturating_sub(scroll.x).max(area.x.into()),
            y: self.end.y.saturating_sub(scroll.y).max(area.y.into()),
        };

        let start_point = rect::Point {
            x: (screen_start.x.min(area.width as usize) as u16),
            y: (screen_start.y.min(area.height as usize) as u16),
        };

        let end_point = rect::Point {
            x: (screen_end.x.min(area.width as usize) as u16),
            y: (screen_end.y.min(area.height as usize) as u16),
        };

        start_point..end_point
    }
}

#[derive(Debug, Clone)]
pub struct Window {
    pub id: WindowId,
    pub document: DocumentId,
    // current text selection defined as start/end points, this should be updated
    // whenever the cursor moves on the window, and cleared when any action that
    // involves the selection is performed
    selection: Selection,
    scroll: Point,
    pub area: Rect,
}

impl Window {
    pub fn new(document: DocumentId) -> Window {
        Window {
            document,
            area: Rect::default(),
            id: WindowId::default(),
            scroll: Point::default(),
            selection: Default::default(),
        }
    }

    pub fn selection(&self) -> Selection {
        self.selection
    }

    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = selection;
    }

    pub fn scroll(&self) -> Point {
        self.scroll
    }

    pub fn scroll_left(&mut self) {
        self.scroll.x = self.scroll.x.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll.y += 1;
    }

    pub fn scroll_up(&mut self) {
        self.scroll.y = self.scroll.y.saturating_sub(1);
    }

    pub fn scroll_right(&mut self) {
        self.scroll.x += 1;
    }

    pub fn scroll_y_to(&mut self, to: usize) {
        self.scroll.y = to;
    }
}
