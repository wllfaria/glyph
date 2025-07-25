use std::fmt::Debug;

use crate::buffer_manager::Buffer;
use crate::geometry::Point;
use crate::key_mapper::EditorMode;

pub struct StatuslineContext<'ctx> {
    pub current_mode: EditorMode,
    pub cursor_position: Point<usize>,
    pub buffer_info: &'ctx Buffer,
    pub width: usize,
}

pub trait StatuslineProvider: Debug {
    fn render_statusline(&self, ctx: &StatuslineContext) -> String;
}
