use crate::{
    config::{Config, LineNumbers},
    theme::Theme,
    tui::{
        gutter::{
            absolute_line_gutter::AbsoluteLineGutter, noop_line_gutter::NoopLineDrawer,
            relative_line_gutter::RelativeLineDrawer,
        },
        rect::Rect,
    },
};

use super::Frame;

pub mod absolute_line_gutter;
pub mod noop_line_gutter;
pub mod relative_line_gutter;

pub enum GutterKind<'a> {
    Relative(RelativeLineDrawer<'a>),
    Absolute(AbsoluteLineGutter<'a>),
    Noop(NoopLineDrawer<'a>),
}

impl GutterKind<'_> {
    pub fn width(&self) -> u16 {
        match self {
            GutterKind::Relative(g) => g.width(),
            GutterKind::Absolute(g) => g.width(),
            GutterKind::Noop(g) => g.width(),
        }
    }

    pub fn render(&self, frame: &mut Frame, total_lines: usize, line: usize, scroll: usize) {
        match self {
            GutterKind::Relative(g) => g.draw(frame, total_lines, line, scroll),
            GutterKind::Absolute(g) => g.draw(frame, total_lines, line, scroll),
            GutterKind::Noop(g) => g.draw(frame, total_lines, line, scroll),
        }
    }
}

pub trait Gutter: std::fmt::Debug {
    fn draw(&self, viewport: &mut Frame, total_lines: usize, line: usize, scroll: usize);
    fn width(&self) -> u16;
}

pub fn get_gutter<'a>(config: &'a Config, theme: &'a Theme, area: Rect) -> GutterKind<'a> {
    match config.line_numbers {
        LineNumbers::Relative => GutterKind::Relative(RelativeLineDrawer::new(config, theme, area)),
        LineNumbers::RelativeNumbered => {
            GutterKind::Relative(RelativeLineDrawer::new(config, theme, area))
        }
        LineNumbers::None => GutterKind::Noop(NoopLineDrawer::new(config, theme, area)),
        LineNumbers::Absolute => GutterKind::Absolute(AbsoluteLineGutter::new(config, theme, area)),
    }
}
