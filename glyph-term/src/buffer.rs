use crate::backend::{Cell, Drawable};
use crate::graphics::Rect;

#[derive(Debug)]
pub struct Buffer {
    area: Rect,
    cells: Vec<Cell>,
}

#[derive(Debug)]
pub struct ChangeSet<'cs> {
    changes: Vec<Drawable<'cs>>,
}

pub struct ChangeSetIter<'cs> {
    iter: std::slice::Iter<'cs, Drawable<'cs>>,
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

    fn idx(&self, x: u16, y: u16) -> u16 {
        y * self.area.width + x
    }

    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell) {
        let idx = self.idx(x, y);
        self.cells[idx as usize] = cell
    }

    #[tracing::instrument(skip(string))]
    pub fn set_string<S: AsRef<str>>(&mut self, x: u16, y: u16, string: S) {
        let idx = self.idx(x, y);
        for (ch_idx, ch) in string.as_ref().chars().enumerate() {
            self.cells[idx as usize + ch_idx] = Cell::new(ch);
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
