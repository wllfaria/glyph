mod crossterm;

use std::io;

pub use crossterm::CrosstermBackend;
use glyph_config::GlyphConfig;
use glyph_core::highlights::HighlightGroup;
use glyph_core::rect::Rect;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub symbol: char,
    pub style: HighlightGroup,
}

#[derive(Debug)]
pub struct Drawable<'a> {
    pub x: u16,
    pub y: u16,
    pub cell: &'a Cell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CursorKind {
    Block,
    Hidden,
}

pub trait Backend {
    fn setup(&mut self) -> Result<(), io::Error>;
    fn restore(&mut self) -> Result<(), io::Error>;
    fn draw<'a, I, T>(&mut self, content: I, config: GlyphConfig) -> Result<(), io::Error>
    where
        I: Iterator<Item = T>,
        T: Into<Drawable<'a>>;
    fn hide_cursor(&mut self) -> Result<(), io::Error>;
    fn show_cursor(&mut self) -> Result<(), io::Error>;
    fn set_cursor(&mut self, x: u16, y: u16, kind: CursorKind) -> Result<(), io::Error>;
    fn area(&self) -> Result<Rect, io::Error>;
    fn flush(&mut self) -> Result<(), io::Error>;
}

impl Cell {
    pub fn new(symbol: char) -> Cell {
        Cell {
            symbol,
            style: HighlightGroup::default(),
        }
    }
}

impl<'a> Drawable<'a> {
    pub fn new(x: u16, y: u16, cell: &'a Cell) -> Drawable<'a> {
        Drawable { x, y, cell }
    }
}

impl<'a> From<(u16, u16, &'a Cell)> for Drawable<'a> {
    fn from((x, y, cell): (u16, u16, &'a Cell)) -> Drawable<'a> {
        Drawable::new(x, y, cell)
    }
}
