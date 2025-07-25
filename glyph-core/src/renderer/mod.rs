use crate::buffer_manager::Buffer;
use crate::geometry::Size;
use crate::key_mapper::EditorMode;
use crate::renderer::error::Result;
use crate::status_provider::StatuslineProvider;
use crate::view_manager::{LayoutTreeNode, ViewManager};

pub mod error;

#[derive(Debug)]
pub struct RenderContext<'ctx> {
    pub mode: EditorMode,
    pub views: &'ctx ViewManager,
    pub layout: &'ctx LayoutTreeNode,
    pub buffers: &'ctx [&'ctx Buffer],
    pub statusline_provider: &'ctx dyn StatuslineProvider,
}

pub trait Renderer {
    fn render(&mut self, ctx: &mut RenderContext<'_>) -> Result<()>;
    fn get_size(&self) -> Result<Size>;
    fn resize(&mut self, size: Size) -> Result<()>;
    fn setup(&self) -> Result<()>;
    fn shutdown(&self) -> Result<()>;
}
