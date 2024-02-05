mod absolute_line_drawer;
mod line_drawer;
mod relative_line_drawer;

use line_drawer::LineDrawer;

pub mod pane;
pub mod pane_dimension;

pub use pane::Pane;
pub use pane_dimension::PaneDimensions;

#[derive(Debug)]
pub struct Position {
    pub row: u16,
    pub col: u16,
}

impl Default for Position {
    fn default() -> Self {
        Self { row: 0, col: 0 }
    }
}
