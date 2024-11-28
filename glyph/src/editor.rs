use std::collections::BTreeMap;
use std::path::PathBuf;

use glyph_term::graphics::Rect;

use crate::document::{Document, DocumentId};
use crate::tab::Tab;
use crate::tree::Layout;
use crate::window::Window;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

#[derive(Debug)]
pub struct Editor {
    mode: Mode,
    next_document_id: DocumentId,
    documents: BTreeMap<DocumentId, Document>,
    focused_tab: usize,
    tabs: Vec<Tab>,
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
        }
    }

    pub fn document(&self, id: &DocumentId) -> Option<&Document> {
        self.documents.get(id)
    }

    pub fn get_focused_tab(&self) -> &Tab {
        &self.tabs[self.focused_tab]
    }

    fn get_focused_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.focused_tab]
    }

    #[must_use]
    pub fn should_close(&self) -> bool {
        self.get_focused_tab().tree.is_empty()
    }

    pub fn new_file(&mut self, action: OpenAction) -> DocumentId {
        let mut document = Document::default();
        let id = self.next_document_id;
        document.id = id;
        self.next_document_id = self.next_document_id.next();
        self.documents.insert(id, document);
        self.switch_document(id, action);

        id
    }

    pub fn new_file_with_document(&mut self, path: PathBuf, text: String, action: OpenAction) -> DocumentId {
        let mut document = Document::new(Some(path), Some(text));
        let id = self.next_document_id;
        document.id = id;
        self.next_document_id = self.next_document_id.next();
        self.documents.insert(id, document);
        self.switch_document(id, action);

        id
    }

    pub fn switch_document(&mut self, id: DocumentId, action: OpenAction) {
        if !self.documents.contains_key(&id) {
            return;
        }

        match action {
            OpenAction::Replace => todo!(),
            OpenAction::SplitVertical => {
                let tab = self.get_focused_tab_mut();

                // get the current focused window or make a new one if theres none
                let mut window = tab
                    .tree
                    .try_get(tab.tree.focus)
                    .filter(|w| id == w.document)
                    .cloned()
                    .unwrap_or_else(|| Window::new(id));

                window.document = id;

                tab.tree.split(window, Layout::Vertical);
            }
            OpenAction::SplitHorizontal => todo!(),
        }
    }
}
