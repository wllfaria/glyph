use crossterm::style::Color;
use tree_sitter::{Language, Parser, Query, QueryCursor};
use tree_sitter_rust::HIGHLIGHT_QUERY;

pub struct Highlight {
    parser: Parser,
    language: Language,
}

#[derive(Debug)]
pub struct ColorInfo {
    pub start: usize,
    pub end: usize,
    pub color: Color,
}

impl Highlight {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::language();
        parser.set_language(language).expect("rust grammar");
        Self { parser, language }
    }

    pub fn colors(&mut self, buffer: &str) -> Vec<ColorInfo> {
        let tree = self.parser.parse(buffer, None).unwrap();
        let query = Query::new(self.language, HIGHLIGHT_QUERY).expect("rust highlight");

        let mut colors = Vec::new();
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), buffer.as_bytes());

        for m in matches {
            for cap in m.captures {
                let node = cap.node;
                let start = node.start_byte();
                let end = node.end_byte();
                let capture_name = query.capture_names()[cap.index as usize].as_str();
                let color = match capture_name {
                    "string" => Color::Red,
                    "function" => Color::Blue,
                    "keyword" => Color::Green,
                    "return" => Color::Magenta,
                    "struct" => Color::Cyan,
                    _ => Color::White,
                };
                colors.push(ColorInfo { start, end, color });
            }
        }

        colors
    }
}
