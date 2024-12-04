use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Parser, Point, Query, Tree};

use crate::document::{Document, DocumentId, LanguageId};

#[derive(Debug)]
pub struct SyntaxCapture {
    pub start: Point,
    pub end: Point,
    pub name: String,
}

#[derive(Debug)]
pub struct Syntax {
    pub language: LanguageId,
    /// document in which this syntax was applied
    pub document: DocumentId,
    /// tree representation of the document
    pub tree: Option<Tree>,
    /// map of captures where each key represents a line
    pub captures: HashMap<usize, Vec<SyntaxCapture>>,
}

#[derive(Default)]
pub struct Highlighter {
    queries: HashMap<LanguageId, Query>,
    parsers: HashMap<LanguageId, Parser>,
    trees: BTreeMap<DocumentId, Syntax>,
}

impl std::fmt::Debug for Highlighter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Highlighter")
            .field("queries", &self.queries)
            .field("trees", &self.trees)
            .field("parsers", &self.parsers.len())
            .finish()
    }
}

impl Highlighter {
    pub fn new() -> Highlighter {
        Highlighter {
            trees: Default::default(),
            parsers: Default::default(),
            queries: Default::default(),
        }
    }

    pub fn document_syntax(&self, document_id: DocumentId) -> Option<&Syntax> {
        self.trees.get(&document_id)
    }

    pub fn add_document(&mut self, document: &Document) {
        let parser = self.get_or_create_parser(document.language());
        let tree = parser.and_then(|p| p.parse(document.text().slice(..).to_string(), None));

        let mut syntax = Syntax {
            language: document.language(),
            document: document.id,
            captures: Default::default(),
            tree,
        };

        if let Some(ref tree) = syntax.tree {
            let mut cursor = tree_sitter::QueryCursor::new();
            let root = tree.root_node();
            let language = self.get_ts_language(document.language()).unwrap();
            let query = tree_sitter::Query::new(&language.into(), tree_sitter_rust::HIGHLIGHTS_QUERY).unwrap();
            let text = document.text().slice(..).to_string();
            let mut matches = cursor.matches(&query, root, text.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    let node = capture.node;
                    let start = node.start_position();
                    let end = node.end_position();
                    let name = query.capture_names()[capture.index as usize].to_string();
                    let entry = syntax.captures.entry(start.row).or_default();
                    let capture = SyntaxCapture { start, end, name };
                    entry.push(capture);
                }
            }
        }

        self.trees.insert(document.id, syntax);
    }

    fn get_or_create_parser(&mut self, language: LanguageId) -> Option<&mut Parser> {
        if self.parsers.contains_key(&language) {
            return self.parsers.get_mut(&language);
        }

        if let Some(ts_language) = self.get_ts_language(language) {
            let mut parser = Parser::new();
            parser.set_language(&ts_language.into()).ok();
            self.parsers.insert(language, parser);
            return self.parsers.get_mut(&language);
        }

        None
    }

    fn get_ts_language(&self, language: LanguageId) -> Option<impl Into<Language>> {
        match language {
            LanguageId::Rust => Some(tree_sitter_rust::LANGUAGE),
            _ => todo!(),
        }
    }
}
