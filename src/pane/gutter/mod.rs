use super::Frame;

pub mod absolute_line_gutter;
pub mod noop_line_gutter;
pub mod relative_line_gutter;

pub trait Gutter: std::fmt::Debug {
    fn draw(&self, viewport: &mut Frame, total_lines: usize, line: usize, scroll: usize);
}
