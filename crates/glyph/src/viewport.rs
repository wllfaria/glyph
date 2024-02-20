use crate::theme::{Style, Theme};

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub c: char,
    pub style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            style: Theme::get().style.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Viewport {
    pub cells: Vec<Cell>,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug)]
pub struct Change<'a> {
    pub cell: &'a Cell,
    pub row: usize,
    pub col: usize,
}

impl Viewport {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![Default::default(); width * height],
        }
    }

    pub fn set_cell(&mut self, col: usize, row: usize, c: char, style: &Style) {
        let pos = row * self.width + col;
        self.cells[pos] = Cell {
            c,
            style: style.clone(),
        };
    }

    pub fn set_text(&mut self, col: usize, row: usize, text: &str, style: &Style) {
        let pos = (row * self.width) + col;
        for (i, c) in text.chars().enumerate() {
            self.cells[pos + i] = Cell {
                c,
                style: style.clone(),
            }
        }
    }

    pub fn diff(&self, other: &Viewport) -> Vec<Change> {
        let mut changes = vec![];
        for (p, cell) in self.cells.iter().enumerate() {
            if *cell != other.cells[p] {
                let row = p / self.width;
                let col = p % self.width;

                changes.push(Change { row, col, cell });
            }
        }
        changes
    }
}
