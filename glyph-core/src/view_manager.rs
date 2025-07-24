use std::collections::BTreeMap;

use crate::buffer_manager::BufferId;
use crate::cursor::Cursor;
use crate::geometry::{Point, Rect, Size};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewId(u64);

impl ViewId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<u64> for ViewId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<u32> for ViewId {
    fn from(id: u32) -> Self {
        Self(id as u64)
    }
}

impl From<u16> for ViewId {
    fn from(id: u16) -> Self {
        Self(id as u64)
    }
}

impl From<u8> for ViewId {
    fn from(id: u8) -> Self {
        Self(id as u64)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct View {
    pub id: ViewId,
    pub buffer_id: BufferId,
    pub scroll_offset: Point<usize>,
    pub cursors: Vec<Cursor>,
}

impl View {
    pub fn new(id: ViewId, buffer_id: BufferId) -> Self {
        Self {
            id,
            buffer_id,
            scroll_offset: Point::default(),
            cursors: vec![Cursor::default()],
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct LeafView {
    pub view_id: ViewId,
    pub rect: Rect,
}

impl LeafView {
    pub fn accept<V>(&self, visitor: &mut V)
    where
        V: LayoutTreeVisitor,
    {
        visitor.visit_leaf(self);
    }
}

#[derive(Debug)]
pub struct SplitView {
    pub rect: Rect,
    pub children: Vec<LayoutTreeNode>,
}

impl SplitView {
    pub fn accept<V>(&self, visitor: &mut V)
    where
        V: LayoutTreeVisitor,
    {
        visitor.visit_split(self);

        for child in self.children.iter() {
            child.accept(visitor);
        }
    }
}

#[derive(Debug)]
pub enum LayoutTreeNode {
    Leaf(LeafView),
    Split(SplitView),
}

impl LayoutTreeNode {
    pub fn accept<V>(&self, visitor: &mut V)
    where
        V: LayoutTreeVisitor,
    {
        match self {
            LayoutTreeNode::Leaf(leaf) => leaf.accept(visitor),
            LayoutTreeNode::Split(split) => split.accept(visitor),
        }
    }
}

pub trait LayoutTreeVisitor {
    fn visit_split(&mut self, split: &SplitView) {
        _ = split;
    }

    fn visit_leaf(&mut self, leaf: &LeafView) {
        _ = leaf;
    }
}

struct ViewCollector<'a> {
    views: &'a mut Vec<ViewId>,
}

impl LayoutTreeVisitor for ViewCollector<'_> {
    fn visit_leaf(&mut self, leaf: &LeafView) {
        self.views.push(leaf.view_id);
    }
}

#[derive(Debug)]
pub struct ViewManager {
    next_view_id: ViewId,
    pub(crate) views: BTreeMap<ViewId, View>,
    pub(crate) layout: LayoutTreeNode,
    pub(crate) active_view: ViewId,
}

#[derive(Debug)]
struct LayoutFinder {
    needle: ViewId,
    leaf: LeafView,
}

impl LayoutFinder {
    pub fn new(needle: ViewId, leaf: LeafView) -> Self {
        Self { needle, leaf }
    }
}

impl LayoutTreeVisitor for LayoutFinder {
    fn visit_leaf(&mut self, leaf: &LeafView) {
        if leaf.view_id == self.needle {
            self.leaf = *leaf;
        }
    }
}

impl ViewManager {
    pub fn new(initial_buffer: BufferId, size: impl Into<Size>) -> Self {
        let size = size.into();
        let mut views = BTreeMap::new();
        let view_id = ViewId::new(0);
        let view = View::new(view_id, initial_buffer);
        views.insert(view_id, view);

        let rect: Rect = size.into();

        Self {
            views,
            next_view_id: ViewId::new(1),
            layout: LayoutTreeNode::Leaf(LeafView { view_id, rect }),
            active_view: view_id,
        }
    }

    pub fn get_visible(&self) -> Vec<&View> {
        let mut visible_views_ids = vec![];
        self.layout.accept(&mut ViewCollector {
            views: &mut visible_views_ids,
        });

        visible_views_ids
            .iter()
            .map(|id| self.views.get(id).unwrap())
            .collect()
    }

    pub fn get_active_view_id(&self) -> ViewId {
        self.active_view
    }

    pub fn get_active_view(&self) -> &View {
        self.views
            .get(&self.active_view)
            .expect("editor must have at least one view")
    }

    pub fn get_mut_active_view(&mut self) -> &mut View {
        self.views
            .get_mut(&self.active_view)
            .expect("editor must have at least one view")
    }

    pub fn get_layout_for_view(&self, view_id: ViewId) -> LeafView {
        assert!(self.views.contains_key(&view_id));
        let mut finder = LayoutFinder::new(view_id, LeafView::default());
        self.layout.accept(&mut finder);
        finder.leaf
    }
}
