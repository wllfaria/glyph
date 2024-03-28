use crate::theme::Style;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub c: char,
    pub style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            style: Style::default(),
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
        self.cells[pos] = Cell { c, style: *style };
    }

    pub fn set_text(&mut self, col: usize, row: usize, text: &str, style: &Style) {
        let pos = (row * self.width) + col;
        for (i, c) in text.chars().enumerate() {
            self.cells[pos + i] = Cell { c, style: *style }
        }
    }

    pub fn diff(&self, other: &Viewport) -> Vec<Change> {
        let mut changes = vec![];
        for (p, cell) in self.cells.iter().enumerate() {
            let row = p / self.width;
            let col = p % self.width;

            if other.cells.len() != self.cells.len() {
                changes.push(Change { row, col, cell });
                continue;
            }

            if *cell != other.cells[p] {
                changes.push(Change { row, col, cell });
            }
        }
        changes
    }

    pub fn clear(&mut self) {
        self.cells = vec![Default::default(); self.width * self.height];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Style;

    #[test]
    fn test_initialize_correctly() {
        let vp = Viewport::new(10, 10);

        assert_eq!(vp.cells.len(), 100);
        assert_eq!(vp.width, 10);
        assert_eq!(vp.height, 10);
    }

    #[test]
    fn test_insert_cell() {
        let mut vp = Viewport::new(10, 10);
        let s = Style::default();

        vp.set_cell(10, 3, '!', &s);

        assert_eq!(vp.cells[40].c, '!');
        assert_eq!(vp.cells[40], Cell { c: '!', style: s });
    }

    #[test]
    fn test_insert_text() {
        let mut vp = Viewport::new(10, 10);
        let s = Style::default();

        vp.set_text(10, 3, "Hello, World!", &s);

        assert_eq!(vp.cells[40].c, 'H');
        assert_eq!(vp.cells[41].c, 'e');
        assert_eq!(vp.cells[42].c, 'l');
        assert_eq!(vp.cells[43].c, 'l');
        assert_eq!(vp.cells[44].c, 'o');
        assert_eq!(vp.cells[45].c, ',');
        assert_eq!(vp.cells[46].c, ' ');
        assert_eq!(vp.cells[47].c, 'W');
        assert_eq!(vp.cells[48].c, 'o');
        assert_eq!(vp.cells[49].c, 'r');
        assert_eq!(vp.cells[50].c, 'l');
        assert_eq!(vp.cells[51].c, 'd');
        assert_eq!(vp.cells[52].c, '!');
    }

    #[test]
    #[should_panic]
    fn test_insert_cell_out_of_bounds() {
        let mut vp = Viewport::new(10, 10);
        let s = Style::default();

        vp.set_cell(11, 11, '!', &s);
    }

    #[test]
    #[should_panic]
    fn test_insert_text_out_of_bounds() {
        let mut vp = Viewport::new(10, 10);
        let s = Style::default();

        vp.set_text(10, 10, "Hello, World!", &s);
    }

    #[test]
    fn test_clear() {
        let mut vp = Viewport::new(2, 2);
        let s = Style::default();

        vp.set_text(0, 0, "1234", &s);

        assert_eq!(vp.cells[0].c, '1');
        assert_eq!(vp.cells[1].c, '2');
        assert_eq!(vp.cells[2].c, '3');
        assert_eq!(vp.cells[3].c, '4');

        vp.clear();

        assert_eq!(vp.cells[0].c, ' ');
        assert_eq!(vp.cells[1].c, ' ');
        assert_eq!(vp.cells[2].c, ' ');
        assert_eq!(vp.cells[3].c, ' ');
    }

    #[test]
    fn test_diff() {
        let mut one = Viewport::new(2, 2);
        let mut two = Viewport::new(2, 2);
        let s = Style::default();

        one.set_text(0, 0, "1234", &s);
        two.set_text(0, 0, "4321", &s);

        assert!(one.diff(&two).len() == 4);
    }
}
