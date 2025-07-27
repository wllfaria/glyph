use crate::buffer_manager::Buffer;
use crate::editing_plugin::EditingPlugin;
use crate::geometry::Size;
use crate::key_mapper::EditorMode;
use crate::renderer::error::Result;
use crate::view_manager::{LayoutTreeNode, ViewManager};

pub mod error;

#[derive(Debug)]
pub struct RenderContext<'ctx> {
    pub mode: EditorMode,
    pub views: &'ctx ViewManager,
    pub layout: &'ctx LayoutTreeNode,
    pub buffers: &'ctx [&'ctx Buffer],
    pub editing_plugin: &'ctx dyn EditingPlugin,
}

pub trait Renderer {
    fn render(&mut self, ctx: &mut RenderContext<'_>) -> Result<()>;
    fn get_size(&self, dock_height: u16) -> Result<Size>;
    fn resize(&mut self, size: Size) -> Result<()>;
    fn setup(&self) -> Result<()>;
    fn shutdown(&self) -> Result<()>;
}
