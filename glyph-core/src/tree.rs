use std::collections::BTreeMap;

use crate::rect::Rect;
use crate::window::{Window, WindowId};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Layout {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub struct Tree {
    area: Rect,
    root: WindowId,
    focus: WindowId,
    next_window: WindowId,
    nodes: BTreeMap<WindowId, Node>,
}

#[derive(Debug)]
pub struct Node {
    parent: WindowId,
    pub value: NodeValue,
}

impl Node {
    pub fn window(window: Window) -> Node {
        Node {
            parent: WindowId::default(),
            value: NodeValue::Window(Box::new(window)),
        }
    }

    pub fn split(layout: Layout) -> Node {
        Node {
            parent: WindowId::default(),
            value: NodeValue::Split(Box::new(Split::new(layout))),
        }
    }
}

#[derive(Debug)]
pub enum NodeValue {
    Window(Box<Window>),
    Split(Box<Split>),
}

#[derive(Debug)]
pub struct Split {
    pub layout: Layout,
    pub nodes: Vec<WindowId>,
    pub area: Rect,
}

impl Split {
    pub fn new(layout: Layout) -> Split {
        Split {
            layout,
            nodes: Vec::default(),
            area: Rect::default(),
        }
    }
}

impl Tree {
    pub fn new(area: Rect) -> Tree {
        Tree {
            root: WindowId::default(),
            focus: WindowId::default(),
            nodes: BTreeMap::default(),
            next_window: WindowId::default(),
            area,
        }
    }

    pub fn focus(&self) -> WindowId {
        self.focus
    }

    pub fn nodes(&self) -> &BTreeMap<WindowId, Node> {
        &self.nodes
    }

    pub fn windows(&self) -> impl Iterator<Item = (&Window, bool)> {
        self.nodes.iter().filter_map(|(key, node)| match node {
            Node {
                value: NodeValue::Window(window),
                ..
            } => Some((window.as_ref(), &self.focus == key)),
            _ => None,
        })
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn get_window(&self, id: WindowId) -> Option<&Window> {
        match self.nodes.get(&id) {
            Some(Node {
                value: NodeValue::Window(window),
                ..
            }) => Some(window),
            _ => None,
        }
    }

    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        match self.nodes.get_mut(&id) {
            Some(Node {
                value: NodeValue::Window(window),
                ..
            }) => Some(window),
            _ => None,
        }
    }

    pub fn window(&self, id: WindowId) -> &Window {
        self.get_window(id).unwrap()
    }

    pub fn window_mut(&mut self, id: WindowId) -> &mut Window {
        self.get_window_mut(id).unwrap()
    }

    pub fn split(&mut self, mut window: Window, _layout: Layout) -> WindowId {
        if self.nodes.is_empty() {
            window.area = self.area.with_height(self.area.height - 2);

            let node = Node::window(window);
            let id = self.next_window;
            self.next_window = self.next_window.next();
            self.nodes.insert(id, node);

            self.window_mut(id).id = id;
            // root is its own parent
            self.nodes.get_mut(&id).unwrap().parent = id;
            self.focus = id;
            self.root = id;
            return id;
        }

        todo!();
    }
}
