use tree_sitter::{Parser, Query, QueryCursor};
use tree_sitter_rust::{language, HIGHLIGHT_QUERY};

use crate::theme::{Style, Theme};

pub struct Highlight {
    parser: Parser,
    query: Query,
    theme: &'static Theme,
}

#[derive(Debug)]
pub struct ColorInfo<'a> {
    pub start: usize,
    pub end: usize,
    pub style: &'a Style,
}

impl Highlight {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language = language();
        parser.set_language(language).expect("rust grammar");
        let query = Query::new(language, HIGHLIGHT_QUERY).expect("rust highlight");
        let theme = Theme::get();

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
        logger::debug!("render buffer with len {}", buffer.len());

        for m in matches {
            for cap in m.captures {
                let node = cap.node;
                let start = node.start_byte();
                let end = node.end_byte();
                let capture_name = self.query.capture_names()[cap.index as usize].as_str();
                if let Some(style) = self.theme.tokens.get(capture_name) {
                    colors.push(ColorInfo { start, end, style });
                } else {
                    logger::info!("missing capture name: {capture_name}");
                }
            }
        }

        colors
    }
}
