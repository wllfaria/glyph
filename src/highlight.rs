use std::fmt::{self, Debug, Formatter};

use tree_sitter::{Parser, Query, QueryCursor};
use tree_sitter_rust::{language, HIGHLIGHT_QUERY};

use crate::theme::{Style, Theme};

pub struct Highlight<'a> {
    parser: Parser,
    query: Query,
    theme: &'a Theme,
}

impl Debug for Highlight<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Highlight")
            .field("query", &self.query)
            .field("theme", &self.theme)
            .finish()
    }
}

#[derive(Debug)]
pub struct ColorInfo<'a> {
    pub start: usize,
    pub end: usize,
    pub style: &'a Style,
}

impl<'a> Highlight<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        let mut parser = Parser::new();
        let language = language();
        parser.set_language(language).expect("rust grammar");
        let query = Query::new(language, HIGHLIGHT_QUERY).expect("rust highlight");

        Self {
            parser,
            query,
            theme,
        }
    }

    pub fn colors(&mut self, buffer: &str) -> Vec<ColorInfo> {
        let tree = self.parser.parse(buffer, None).unwrap();

        let mut colors = Vec::new();
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), buffer.as_bytes());

        for m in matches {
            for cap in m.captures {
                let node = cap.node;
                let start = node.start_byte();
                let end = node.end_byte();
                let capture_name = self.query.capture_names()[cap.index as usize].as_str();
                if let Some(style) = self.theme.tokens.get(capture_name) {
                    colors.push(ColorInfo { start, end, style });
                }
            }
        }

        colors
    }
}
