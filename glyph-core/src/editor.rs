use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::document::{Document, DocumentId};
use crate::rect::Rect;
use crate::tab::Tab;
use crate::tree::Layout;
use crate::window::{Window, WindowId};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum EventResult {
    Consumed(Option<usize>),
    Ignored(Option<usize>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Normal => f.write_str("normal"),
            Mode::Insert => f.write_str("insert"),
        }
    }
}

#[derive(Debug)]
pub struct Editor {
    mode: Mode,
    next_document_id: DocumentId,
    documents: BTreeMap<DocumentId, Document>,
    pub focused_tab: usize,
    pub tabs: Vec<Tab>,
    area: Rect,
}

#[derive(Debug)]
pub enum OpenAction {
    Replace,
    SplitVertical,
    SplitHorizontal,
}

impl Editor {
    pub fn new(area: Rect) -> Editor {
        Editor {
            // first document id will be 1 by default
            next_document_id: DocumentId::default(),
            mode: Mode::Normal,
            documents: BTreeMap::default(),
            tabs: vec![Tab::new(area)],
            focused_tab: 0,
            area,
        }
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn get_document(&self, id: &DocumentId) -> Option<&Document> {
        self.documents.get(id)
    }

    pub fn document(&self, id: &DocumentId) -> &Document {
        self.get_document(id).unwrap()
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn focused_tab(&self) -> &Tab {
        &self.tabs[self.focused_tab]
    }

    pub fn focused_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.focused_tab]
    }

    #[must_use]
    pub fn should_close(&self) -> bool {
        self.focused_tab().tree.is_empty()
    }

    pub fn new_file(&mut self, action: OpenAction) -> (WindowId, DocumentId) {
        let mut document = Document::default();
        let id = self.next_document_id;
        document.id = id;
        self.next_document_id = self.next_document_id.next();
        self.documents.insert(id, document);
        let window_id = self.switch_document(id, action);

        (window_id, id)
    }

    pub fn new_file_with_document(
        &mut self,
        path: PathBuf,
        text: String,
        action: OpenAction,
    ) -> (WindowId, DocumentId) {
        let mut document = Document::new(Some(path), Some(text));
        let id = self.next_document_id;
        document.id = id;
        self.next_document_id = self.next_document_id.next();
        self.documents.insert(id, document);
        let window_id = self.switch_document(id, action);

        (window_id, id)
    }

    pub fn switch_document(&mut self, id: DocumentId, action: OpenAction) -> WindowId {
        assert!(self.documents.contains_key(&id));

        match action {
            OpenAction::Replace => todo!(),
            OpenAction::SplitVertical => {
                let tab = self.focused_tab_mut();

                // get the current focused window or make a new one if theres none
                let mut window = tab
                    .tree
                    .get_window(tab.tree.focus())
                    .filter(|w| id == w.document)
                    .cloned()
                    .unwrap_or_else(|| Window::new(id));

                window.document = id;

                tab.tree.split(window, Layout::Vertical)
            }
            OpenAction::SplitHorizontal => todo!(),
        }
    }
}
