use glyph_term::graphics::Rect;
use slotmap::HopSlotMap;

use crate::window::{Window, WindowId};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Layout {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub struct Tree {
    root: WindowId,
    pub focus: WindowId,
    area: Rect,
    nodes: HopSlotMap<WindowId, Node>,
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
            nodes: HopSlotMap::with_key(),
            area,
        }
    }

    pub fn windows(&self) -> impl Iterator<Item = (&Window, bool)> {
        self.nodes.iter().filter_map(|(key, node)| match node {
            Node {
                value: NodeValue::Window(window),
                ..
            } => Some((window.as_ref(), self.focus == key)),
            _ => None,
        })
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn try_get(&self, id: WindowId) -> Option<&Window> {
        match self.nodes.get(id) {
            Some(Node {
                value: NodeValue::Window(window),
                ..
            }) => Some(window),
            _ => None,
        }
    }

    pub fn try_get_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        match self.nodes.get_mut(id) {
            Some(Node {
                value: NodeValue::Window(window),
                ..
            }) => Some(window),
            _ => None,
        }
    }

    pub fn get(&mut self, id: WindowId) -> &Window {
        self.try_get(id).unwrap()
    }

    pub fn get_mut(&mut self, id: WindowId) -> &mut Window {
        self.try_get_mut(id).unwrap()
    }

    pub fn split(&mut self, mut window: Window, layout: Layout) -> WindowId {
        if self.nodes.is_empty() {
            window.area = self.area;

            let node = Node::window(window);
            let node = self.nodes.insert(node);

            self.get_mut(node).id = node;
            // root is its own parent
            self.nodes[node].parent = node;
            self.focus = node;
            self.root = node;
            return node;
        }

        todo!();
    }
}
