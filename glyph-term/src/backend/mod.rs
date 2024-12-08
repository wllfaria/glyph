mod crossterm;

use std::io;

pub use crossterm::CrosstermBackend;
use glyph_config::GlyphConfig;
use glyph_core::color::Color;
use glyph_core::highlights::HighlightGroup;
use glyph_core::rect::Rect;

/// how to merge styles on a cell
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum StyleMerge {
    /// Keep original styles if colors are not Reset, or attributes are not false
    Keep,
    /// Replace every style with the given one, even previously set ones
    #[default]
    Replace,
    /// Merge styles that are defined on the given style, even if that replace existing ones this
    /// strategy won't replace a defined style with a Reset color, which differs from Replace
    Merge,
}

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

    pub fn with_style(self, style: HighlightGroup) -> Cell {
        Cell {
            symbol: self.symbol,
            style,
        }
    }

    pub fn merge_style(&mut self, style: HighlightGroup, behavior: StyleMerge) {
        match behavior {
            StyleMerge::Keep => {
                if self.style.fg == Color::Reset {
                    self.style.fg = style.fg;
                }
                if self.style.bg == Color::Reset {
                    self.style.bg = style.bg;
                }
                if !self.style.bold {
                    self.style.bold = style.bold;
                }
            }
            StyleMerge::Replace => self.style = style,
            StyleMerge::Merge => {
                if style.fg != Color::Reset {
                    self.style.fg = style.fg;
                }
                if style.bg != Color::Reset {
                    self.style.bg = style.bg;
                }
                if style.bold {
                    self.style.bold = style.bold;
                }
            }
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
