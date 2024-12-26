use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::document::{Document, DocumentId};
use crate::rect::Rect;
use crate::tab::Tab;
use crate::tree::{CloseAction, Layout};
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
    Command,
}

impl<T> From<T> for Mode
where
    T: AsRef<str>,
{
    fn from(value: T) -> Mode {
        match value.as_ref() {
            "normal" | "n" => Mode::Normal,
            "insert" | "i" => Mode::Insert,
            "command" | "c" => Mode::Command,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Normal => f.write_str("normal"),
            Mode::Insert => f.write_str("insert"),
            Mode::Command => f.write_str("command"),
        }
    }
}

#[derive(Debug)]
pub struct Editor {
    mode: Mode,
    next_document_id: DocumentId,
    documents: BTreeMap<DocumentId, Document>,
    pub messages: String,
    pub command: String,
    pub buffered_keymap: String,
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
    pub fn new(area: impl Into<Rect>) -> Editor {
        let area = area.into();
        Editor {
            // first document id will be 1 by default
            next_document_id: DocumentId::default(),
            mode: Mode::Normal,
            documents: BTreeMap::default(),
            buffered_keymap: String::default(),
            command: String::default(),
            messages: String::default(),
            tabs: vec![Tab::new(area)],
            focused_tab: 0,
            area,
        }
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn get_document(&self, id: DocumentId) -> Option<&Document> {
        self.documents.get(&id)
    }

    pub fn get_document_mut(&mut self, id: DocumentId) -> Option<&mut Document> {
        self.documents.get_mut(&id)
    }

    pub fn document(&self, id: DocumentId) -> &Document {
        self.get_document(id).unwrap()
    }

    pub fn document_mut(&mut self, id: DocumentId) -> &mut Document {
        self.get_document_mut(id).unwrap()
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode
    }

    pub fn focused_tab(&self) -> &Tab {
        &self.tabs[self.focused_tab]
    }

    pub fn focused_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.focused_tab]
    }

    #[must_use]
    pub fn should_close(&self) -> bool {
        self.tabs.len() == 1 && self.focused_tab().tree.is_empty()
    }

    pub fn close_active_window(&mut self) -> bool {
        let tab = self.focused_tab_mut();
        let window = tab.tree.focus();
        match tab.tree.close_window(window) {
            CloseAction::None => false,
            CloseAction::CloseTab => {
                self.tabs.remove(self.focused_tab);
                self.tabs.is_empty()
            }
        }
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
        path: impl Into<PathBuf>,
        text: String,
        action: OpenAction,
    ) -> (WindowId, DocumentId) {
        let mut document = Document::new(Some(path.into()), Some(text));
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
            OpenAction::Replace => {
                let tab = self.focused_tab_mut();
                let window = tab
                    .tree
                    .get_window_mut(tab.tree.focus())
                    .expect("no active window on the editor");

                window.document = id;
                window.id
            }
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

    pub fn write_document(&mut self, document: DocumentId) {
        let document = self.document(document);
        if let Some(path) = document.metadata().path() {
            match crate::fs::write_file(path, document.text()) {
                Ok(amount) => self.messages.push_str(&format!(
                    r#""~/{path}" {lines}L, {amount}B written"#,
                    path = path.display(),
                    lines = document.text().len_lines()
                )),
                Err(_) => todo!(),
            }
        }
    }
}
