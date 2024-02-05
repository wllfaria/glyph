use std::io::Result;

use crate::config::{Config, LineNumbers};
use crate::pane::absolute_line_drawer::AbsoluteLineDrawer;
use crate::pane::noop_line_drawer::NoopLineDrawer;
use crate::pane::pane_dimension::PaneDimensions;
use crate::pane::relative_line_drawer::RelativeLineDrawer;

pub trait LineDrawer: std::fmt::Debug {
    fn draw_lines(
        &mut self,
        dimensions: &PaneDimensions,
        total_lines: u16,
        current_line: u16,
    ) -> Result<()>;
}

impl dyn LineDrawer {
    pub fn get_line_drawer() -> Box<dyn LineDrawer> {
        let config = Config::get();
        match config.line_numbers {
            LineNumbers::Absolute => Box::new(AbsoluteLineDrawer::new()),
            LineNumbers::Relative => Box::new(RelativeLineDrawer::new()),
            LineNumbers::RelativeNumbered => Box::new(RelativeLineDrawer::new()),
            LineNumbers::None => Box::new(NoopLineDrawer::new()),
        }
    }
}
