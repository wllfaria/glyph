use crate::buffer::Buffer;
use crate::theme::Theme;
use crate::viewport::Viewport;
use crate::{pane::Rect, viewport::Cell};

pub struct HoverPopup<'a> {
    area: Rect,
    content: Buffer,
    theme: &'a Theme,
}

impl<'a> HoverPopup<'a> {
    pub fn new(col: usize, row: usize, theme: &'a Theme, content: String) -> Self {
        let buffer = Buffer::from_string(0, &content, 0);
        let area = HoverPopup::calculate_area(&buffer, col, row);
        Self {
            theme,
            content: buffer,
            area: area.clone(),
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

    pub fn render(&mut self, view: &mut Viewport) -> anyhow::Result<()> {
        let cells = self.content_to_vec_cells();
        let mut col = self.area.col;
        let mut row = self.area.row;
        for line in cells.iter() {
            for cell in line {
                match cell.c {
                    '\n' => view.set_cell(col, row, ' ', &cell.style),
                    _ => view.set_cell(col, row, cell.c, &cell.style),
                }
                col += 1;
            }
            col = self.area.col;
            row += 1;
        }
        Ok(())
    }

    fn content_to_vec_cells(&self) -> Vec<Vec<Cell>> {
        let style = self.theme.float;
        let mut cells = vec![vec![Cell { c: ' ', style }; self.area.width + 1]; self.area.height];
        for (i, line) in self.content.lines().enumerate() {
            for (j, c) in line.iter().enumerate() {
                cells[i][j] = Cell { c: *c, style };
            }
        }
        cells
    }
}
