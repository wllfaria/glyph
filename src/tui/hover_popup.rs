use crate::buffer::Buffer;
use crate::theme::Theme;
use crate::viewport::Viewport;
use crate::{pane::Rect, viewport::Cell};

pub struct HoverPopup<'a> {
    _area: Rect,
    _content: Buffer,
    _theme: &'a Theme,
}

impl<'a> HoverPopup<'a> {
    pub fn new(col: usize, row: usize, theme: &'a Theme, content: String) -> Self {
        let buffer = Buffer::from_string(0, &content, 0);
        let area = HoverPopup::calculate_area(&buffer, col, row);
        Self {
            _theme: theme,
            _content: buffer,
            _area: area.clone(),
        }
    }

    fn calculate_area(buffer: &Buffer, col: usize, row: usize) -> Rect {
        let lines = buffer.marker.len();
        let height = lines.min(30);
        let mut width = 0;

        for line in buffer.lines() {
            width = width.max(line.len());
        }

        Rect {
            height,
            width,
            row,
            col,
        }
    }

    pub fn _render(&mut self, view: &mut Viewport) -> anyhow::Result<()> {
        let cells = self._content_to_vec_cells();
        let mut col = self._area.col;
        let mut row = self._area.row;
        for line in cells.iter() {
            for cell in line {
                match cell.c {
                    '\n' => view.set_cell(col, row, ' ', &cell.style),
                    _ => view.set_cell(col, row, cell.c, &cell.style),
                }
                col += 1;
            }
            col = self._area.col;
            row += 1;
        }
        Ok(())
    }

    fn _content_to_vec_cells(&self) -> Vec<Vec<Cell>> {
        let style = self._theme.float;
        let mut cells = vec![vec![Cell { c: ' ', style }; self._area.width + 1]; self._area.height];
        for (i, line) in self._content.lines().enumerate() {
            for (j, c) in line.iter().enumerate() {
                cells[i][j] = Cell { c: *c, style };
            }
        }
        cells
    }
}
