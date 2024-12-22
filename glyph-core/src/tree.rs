use std::collections::BTreeMap;

use crate::rect::Rect;
use crate::window::{Window, WindowId};

#[derive(Debug)]
pub enum NodeValue {
    Window(WindowId),
    Split(Split),
}

#[derive(Debug)]
pub struct Split {
    pub layout: Layout,
    pub windows: Vec<WindowId>,
    pub area: Rect,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum CloseAction {
    None,
    CloseTab,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Layout {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub struct Tree {
    area: Rect,
    focus: WindowId,
    next_window: WindowId,
    root: NodeValue,
    windows: BTreeMap<WindowId, Window>,
}

impl Split {
    pub fn new(layout: Layout) -> Split {
        Split {
            layout,
            windows: Vec::default(),
            area: Rect::default(),
        }
    }
}

impl Tree {
    pub fn new(area: Rect) -> Tree {
        Tree {
            area,
            root: NodeValue::Window(Default::default()),
            focus: WindowId::default(),
            windows: BTreeMap::default(),
            next_window: WindowId::default(),
        }
    }

    pub fn focus(&self) -> WindowId {
        self.focus
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.windows.get(&id)
    }

    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.get_mut(&id)
    }

    pub fn window(&self, id: WindowId) -> &Window {
        self.get_window(id).unwrap()
    }

    pub fn window_mut(&mut self, id: WindowId) -> &mut Window {
        self.get_window_mut(id).unwrap()
    }

    pub fn windows(&self) -> &BTreeMap<WindowId, Window> {
        &self.windows
    }

    pub fn split(&mut self, mut window: Window, _layout: Layout) -> WindowId {
        if self.windows.is_empty() {
            window.area = self.area.with_height(self.area.height - 2);

            let id = self.next_window;
            window.id = id;
            self.root = NodeValue::Window(id);
            self.windows.insert(id, window);
            self.next_window = self.next_window.next();
            self.focus = id;

            return id;
        }

        todo!();
    }

    pub fn close_window(&mut self, window: WindowId) -> CloseAction {
        match &mut self.root {
            NodeValue::Window(id) => {
                self.windows.remove(id);
                return CloseAction::CloseTab;
            }
            NodeValue::Split(split) => {
                split.windows.retain(|&w| w != window);

                if split.windows.len() == 1 {
                    let window = split.windows.first().copied().unwrap();
                    self.focus = window;
                    self.root = NodeValue::Window(window);
                }
            }
        }

        CloseAction::None
    }
}
