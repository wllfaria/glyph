use glyph_term::graphics::Rect;

use crate::tree::Tree;

#[derive(Debug)]
pub struct Tab {
    pub tree: Tree,
}

impl Tab {
    pub fn new(area: Rect) -> Tab {
        Tab { tree: Tree::new(area) }
    }
}
