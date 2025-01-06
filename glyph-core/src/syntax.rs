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
    pub queries: HashMap<LanguageId, Query>,
    pub parsers: HashMap<LanguageId, Parser>,
    pub trees: BTreeMap<DocumentId, Syntax>,
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

    #[tracing::instrument(skip_all, target = "ADD_DOCUMENT")]
    pub fn add_document(&mut self, document: &Document) {
        let parser = self.get_or_create_parser(document.language());
        let tree = parser.and_then(|p| p.parse(document.text().to_string(), None));

        let mut syntax = Syntax {
            language: document.language(),
            document: document.id,
            captures: Default::default(),
            tree,
        };

        if let Some(ref tree) = syntax.tree {
            let start = std::time::Instant::now();
            tracing::debug!("document {} parsed, getting syntax captures", document.id);

            let mut cursor = tree_sitter::QueryCursor::new();
            let root = tree.root_node();
            let language = get_ts_language(document.language()).expect("if we have a tree, we must have a language");

            tracing::debug!("took {:?}", start.elapsed());

            let start = std::time::Instant::now();

            let query = self.queries.entry(document.language()).or_insert_with(|| {
                tree_sitter::Query::new(
                    &language.into(),
                    get_ts_query(document.language()).expect("if we have a tree, we must have a query"),
                )
                .unwrap()
            });

            tracing::debug!("took {:?}", start.elapsed());

            let start = std::time::Instant::now();
            let text = document.text().to_string();
            let mut matches = cursor.matches(query, root, text.as_bytes());
            tracing::debug!(
                "parsed capture matches for document {}, took {:?}",
                document.id,
                start.elapsed()
            );

            let start = std::time::Instant::now();
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
            tracing::debug!(
                "building highlight map for document {} took {:?}",
                document.id,
                start.elapsed()
            );
        }

        self.trees.insert(document.id, syntax);
    }

    pub fn update_document(&mut self, document: &Document) {
        let syntax = self
            .trees
            .get_mut(&document.id)
            .expect("document is not registered on highlighter");

        let Some(ref tree) = syntax.tree else {
            return;
        };

        let parser = self
            .parsers
            .get_mut(&document.language())
            .expect("document has a syntax tree but there was no parser for it");

        let Some(tree) = parser.parse(document.text().slice(..).to_string(), Some(tree)) else {
            return;
        };

        let query = self
            .queries
            .get(&document.language())
            .expect("document has a syntax tree but there was no query for it");

        let mut cursor = tree_sitter::QueryCursor::new();
        let root = tree.root_node();
        let text = document.text().slice(..).to_string();
        let mut matches = cursor.matches(query, root, text.as_bytes());

        syntax.captures.clear();

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

    fn get_or_create_parser(&mut self, language: LanguageId) -> Option<&mut Parser> {
        if self.parsers.contains_key(&language) {
            return self.parsers.get_mut(&language);
        }

        if let Some(ts_language) = get_ts_language(language) {
            let mut parser = Parser::new();
            parser.set_language(&ts_language.into()).ok();
            self.parsers.insert(language, parser);
            return self.parsers.get_mut(&language);
        }

        None
    }
}

fn get_ts_language(language: LanguageId) -> Option<impl Into<Language>> {
    match language {
        LanguageId::Rust => Some(tree_sitter_rust::LANGUAGE),
        LanguageId::Lua => Some(tree_sitter_lua::LANGUAGE),
        LanguageId::Markdown => Some(tree_sitter_md::LANGUAGE),
        LanguageId::C => Some(tree_sitter_c::LANGUAGE),
        LanguageId::Cpp => Some(tree_sitter_cpp::LANGUAGE),
        LanguageId::Zig => Some(tree_sitter_zig::LANGUAGE),
        LanguageId::Ocaml => Some(tree_sitter_ocaml::LANGUAGE_OCAML),
        LanguageId::Plain => None,
    }
}

fn get_ts_query(language: LanguageId) -> Option<&'static str> {
    match language {
        LanguageId::Rust => Some(include_str!("../../languages/queries/rust/highlights.scm")),
        LanguageId::Lua => Some(include_str!("../../languages/queries/lua/highlights.scm")),
        LanguageId::Markdown => Some(include_str!("../../languages/queries/markdown/highlights.scm")),
        LanguageId::C => Some(include_str!("../../languages/queries/c/highlights.scm")),
        LanguageId::Cpp => Some(include_str!("../../languages/queries/cpp/highlights.scm")),
        LanguageId::Zig => Some(include_str!("../../languages/queries/zig/highlights.scm")),
        LanguageId::Ocaml => Some(include_str!("../../languages/queries/ocaml/highlights.scm")),
        LanguageId::Plain => None,
    }
}
