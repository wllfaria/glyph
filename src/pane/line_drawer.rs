use super::pane_dimension::PaneDimensions;
use std::io::Result;

pub trait LineDrawer {
    fn draw_lines(
        &mut self,
        dimensions: &PaneDimensions,
        total_lines: u16,
        current_line: u16,
    ) -> Result<()>;
}
