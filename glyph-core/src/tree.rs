use std::collections::BTreeMap;

use crate::rect::Rect;
use crate::window::{Window, WindowId};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum NodeValue {
    Window(WindowId),
    Split(Split),
}

pub trait NodeVisitor {
    type Output;

    fn visit_window(&mut self, node: &mut NodeValue, window_id: WindowId) -> Option<Self::Output>;
    fn visit_split(&mut self, split: &mut Split) -> Option<Self::Output>;
}

impl NodeValue {
    pub fn visit<V>(&mut self, visitor: &mut V) -> Option<V::Output>
    where
        V: NodeVisitor,
    {
        match self {
            NodeValue::Window(window_id) => {
                let window_id = *window_id;
                visitor.visit_window(self, window_id)
            }
            NodeValue::Split(split) => visitor.visit_split(split),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Split {
    pub layout: Layout,
    pub nodes: Vec<NodeValue>,
    pub area: Rect,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum CloseAction {
    None,
    CloseTab,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Layout {
    Vertical,
    Horizontal,
}

#[derive(Debug)]
pub struct Tree {
    area: Rect,
    focus: WindowId,
    next_window: WindowId,
    root: NodeValue,
    windows: BTreeMap<WindowId, Window>,
}

struct NodeResizer<'a> {
    area: Rect,
    windows: &'a mut BTreeMap<WindowId, Window>,
}

impl NodeVisitor for NodeResizer<'_> {
    type Output = ();

    fn visit_window(&mut self, _: &mut NodeValue, window_id: WindowId) -> Option<Self::Output> {
        self.windows
            .get_mut(&window_id)
            .expect("layout contains non-existing window")
            .area = self.area;

        None
    }

    fn visit_split(&mut self, split: &mut Split) -> Option<Self::Output> {
        let windows = split.nodes.len();
        let width = self.area.width / windows as u16;
        let height = self.area.height / windows as u16;

        let areas = match split.layout {
            Layout::Vertical => split
                .nodes
                .iter()
                .enumerate()
                .map(|(i, _)| Rect::new(width * i as u16, self.area.y, width, self.area.height))
                .collect::<Vec<_>>(),
            Layout::Horizontal => split
                .nodes
                .iter()
                .enumerate()
                .map(|(i, _)| Rect::new(self.area.x, height * i as u16, self.area.width, height))
                .collect::<Vec<_>>(),
        };

        for (node, area) in split.nodes.iter_mut().zip(areas) {
            let mut resizer = NodeResizer {
                area,
                windows: self.windows,
            };
            node.visit(&mut resizer);
        }

        None
    }
}

struct NodeSplitter<'a> {
    target: WindowId,
    new_window: Window,
    layout: Layout,
    windows: &'a mut BTreeMap<WindowId, Window>,
    next_window: &'a mut WindowId,
}

impl NodeVisitor for NodeSplitter<'_> {
    type Output = WindowId;

    fn visit_window(&mut self, node: &mut NodeValue, window_id: WindowId) -> Option<Self::Output> {
        if window_id != self.target {
            return None;
        }

        // If root is a window, we have no splits and therefore it is also the focused window
        let mut split = Split::new(self.layout);

        let id = *self.next_window;
        self.new_window.id = id;

        // TODO: maybe avoid cloning here, although its very small
        self.windows.insert(id, self.new_window.clone());
        *self.next_window = self.next_window.next();

        let area = self.windows.get(&window_id).unwrap().area;
        // TODO: whenever I refactor this to not have the cyclic dependency, this value should
        // be based on configuration
        split.area = area.with_height(area.height);

        let mut left = split.area;
        let right = match self.layout {
            Layout::Vertical => left.split_right(left.width / 2),
            Layout::Horizontal => left.split_bottom(left.height / 2),
        };

        self.windows.get_mut(&window_id).unwrap().area = left;
        self.windows.get_mut(&id).unwrap().area = right;

        split.nodes.push(NodeValue::Window(window_id));
        split.nodes.push(NodeValue::Window(id));

        *node = NodeValue::Split(split);

        Some(id)
    }

    fn visit_split(&mut self, split: &mut Split) -> Option<Self::Output> {
        let target_idx = split
            .nodes
            .iter()
            .position(|node| matches!(node, NodeValue::Window(id) if *id == self.target));

        if let Some(target_idx) = target_idx {
            let node = &mut split.nodes[target_idx];
            let NodeValue::Window(window_id) = node else {
                unreachable!("already checked that target_idx is a window");
            };
            if self.layout != split.layout {
                let window_id = *window_id;
                return self.visit_window(node, window_id);
            }
            let id = *self.next_window;
            self.new_window.id = id;

            // TODO: maybe avoid cloning here, although its very small
            self.windows.insert(id, self.new_window.clone());
            *self.next_window = self.next_window.next();
            split.nodes.push(NodeValue::Window(id));
            return Some(id);
        }

        for node in split.nodes.iter_mut() {
            if let NodeValue::Split(split) = node {
                if let Some(result) = self.visit_split(split) {
                    return Some(result);
                }
            }
        }

        None
    }
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
            area,
            root: NodeValue::Window(WindowId::default()),
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

    pub fn split(&mut self, mut window: Window, layout: Layout) -> WindowId {
        // tree is initialized empty, and splitting it while empty will just setup the root node
        if self.windows.is_empty() {
            // TODO: whenever I refactor this to not have the cyclic dependency, this value should
            // be based on configuration
            window.area = self.area.with_height(self.area.height - 2);

            let id = self.next_window;
            window.id = id;
            self.root = NodeValue::Window(id);
            self.windows.insert(id, window);
            self.next_window = self.next_window.next();
            self.focus = id;

            return id;
        }

        let mut splitter = NodeSplitter {
            target: self.focus,
            new_window: window,
            layout,
            windows: &mut self.windows,
            next_window: &mut self.next_window,
        };

        let window_id = self.root.visit(&mut splitter).expect("failed to find focused window");

        self.resize(self.area);

        window_id
    }

    pub fn close_window(&mut self, _window: WindowId) -> CloseAction {
        match &mut self.root {
            NodeValue::Window(id) => {
                self.windows.remove(id);
                return CloseAction::CloseTab;
            }
            NodeValue::Split(_) => {}
        }

        CloseAction::None
    }

    pub fn resize(&mut self, mut new_area: Rect) {
        self.area = new_area;
        new_area.split_bottom(2);

        let mut resizer = NodeResizer {
            area: new_area,
            windows: &mut self.windows,
        };

        self.root.visit(&mut resizer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;

    #[test]
    fn test_initialize_tree() {
        let mut tree = Tree::new((10, 10).into());
        let document = Document::new::<String>(None, None);
        let window = Window::new(document.id);
        tree.split(window, Layout::Vertical);

        assert_eq!(tree.window(tree.focus()).area, (8, 10).into());
    }

    #[test]
    fn test_split_vertically() {
        let mut tree = Tree::new((10, 10).into());
        let document = Document::new::<String>(None, None);

        let window = Window::new(document.id);
        tree.split(window, Layout::Vertical);

        let window = Window::new(document.id);
        tree.split(window, Layout::Vertical);

        assert_eq!(tree.window(tree.focus()).area, (8, 5).into());
    }

    #[test]
    fn test_multi_split_vertically() {
        let mut tree = Tree::new((10, 15).into());
        let document = Document::new::<String>(None, None);

        let window = Window::new(document.id);
        tree.split(window, Layout::Vertical);

        let window = Window::new(document.id);
        tree.split(window, Layout::Vertical);

        let window = Window::new(document.id);
        tree.split(window, Layout::Vertical);

        assert!(matches!(tree.root, NodeValue::Split(_)));
        if let NodeValue::Split(split) = &tree.root {
            assert_eq!(split.area, Rect::new(0, 0, 15, 8));
        }
        tree.windows
            .values()
            .enumerate()
            .for_each(|(i, w)| assert_eq!(w.area, Rect::new(5 * i as u16, 0, 5, 8)));
    }
}
