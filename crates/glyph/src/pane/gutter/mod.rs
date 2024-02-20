use crate::config::{Config, LineNumbers};
use crate::pane::gutter::absolute_line_gutter::AbsoluteLineGutter;
use crate::pane::gutter::noop_line_gutter::NoopLineDrawer;
use crate::pane::gutter::relative_line_gutter::RelativeLineDrawer;

use super::Viewport;

mod absolute_line_gutter;
mod noop_line_gutter;
mod relative_line_gutter;

pub trait Gutter: std::fmt::Debug {
    fn draw(&self, viewport: &mut Viewport, total_lines: usize, line: usize, scroll: usize);
}

impl dyn Gutter {
    pub fn get_gutter() -> Box<dyn Gutter> {
        let config = Config::get();
        match config.line_numbers {
            LineNumbers::Absolute => Box::new(AbsoluteLineGutter::new()),
            LineNumbers::Relative => Box::new(RelativeLineDrawer::new()),
            LineNumbers::RelativeNumbered => Box::new(RelativeLineDrawer::new()),
            LineNumbers::None => Box::new(NoopLineDrawer::new()),
        }
    }
}
