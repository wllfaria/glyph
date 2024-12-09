use glyph_core::highlights::HighlightGroup;
use glyph_core::rect::Rect;

use crate::backend::{Cell, Drawable, StyleMerge};

#[derive(Debug)]
pub struct Buffer {
    area: Rect,
    cells: Vec<Cell>,
}

#[derive(Debug)]
pub struct ChangeSet<'cs> {
    changes: Vec<Drawable<'cs>>,
}

impl ChangeSet<'_> {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.changes.len()
    }
}

pub struct ChangeSetIter<'cs> {
    iter: std::slice::Iter<'cs, Drawable<'cs>>,
}

pub struct StyleDef {
    pub behavior: StyleMerge,
    pub style: HighlightGroup,
}

pub trait IntoStyleDef {
    fn into_style_def(self) -> StyleDef;
}

impl IntoStyleDef for HighlightGroup {
    fn into_style_def(self) -> StyleDef {
        StyleDef {
            behavior: StyleMerge::default(),
            style: self,
        }
    }
}

impl<'cs> IntoIterator for ChangeSet<'cs> {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = Drawable<'cs>;

    fn into_iter(self) -> Self::IntoIter {
        self.changes.into_iter()
    }
}

impl<'cs> IntoIterator for &'cs ChangeSet<'cs> {
    type IntoIter = ChangeSetIter<'cs>;
    type Item = &'cs Drawable<'cs>;

    fn into_iter(self) -> Self::IntoIter {
        ChangeSetIter {
            iter: self.changes.iter(),
        }
    }
}

impl<'cs> Iterator for ChangeSetIter<'cs> {
    type Item = &'cs Drawable<'cs>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl Buffer {
    pub fn new(area: Rect) -> Buffer {
        Buffer {
            cells: vec![Cell::default(); (area.width * area.height) as usize],
            area,
        }
    }

    #[inline]
    fn idx(&self, x: u16, y: u16) -> u16 {
        y * self.area.width + x
    }

    #[inline]
    pub fn set_cell(&mut self, x: u16, y: u16, ch: char, style: impl IntoStyleDef) {
        let idx = self.idx(x, y);
        let cell = &mut self.cells[idx as usize];
        let style_def = style.into_style_def();
        cell.merge_style(style_def.style, style_def.behavior);
        cell.symbol = ch;
    }

    pub fn set_string<S: AsRef<str>>(&mut self, x: u16, y: u16, string: S, style: impl IntoStyleDef) {
        let idx = self.idx(x, y);
        let str_ref = string.as_ref();
        let style_def = style.into_style_def();

        for (ch_idx, ch) in str_ref.chars().enumerate() {
            let cell = &mut self.cells[idx as usize + ch_idx];
            cell.merge_style(style_def.style, style_def.behavior);
            cell.symbol = ch;
        }
    }

    pub fn diff(&self, other: &Buffer) -> ChangeSet {
        let mut changes = vec![];

        for (idx, cell) in self.cells.iter().enumerate() {
            if cell != &other.cells[idx] {
                let x = idx % self.area.width as usize;
                let y = idx / self.area.width as usize;
                changes.push(Drawable {
                    cell,
                    x: x as u16,
                    y: y as u16,
                });
            }
        }

        ChangeSet { changes }
    }
}
