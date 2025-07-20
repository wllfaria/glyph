use crate::buffer_manager::Buffer;
use crate::geometry::Size;
use crate::renderer::error::Result;
use crate::view_manager::{LayoutTreeNode, View};

pub mod error;

#[derive(Debug)]
pub struct RenderContext<'ctx> {
    pub views: &'ctx [&'ctx View],
    pub layout: &'ctx LayoutTreeNode,
    pub buffers: &'ctx [&'ctx Buffer],
}

pub trait Renderer {
    fn render(&mut self, ctx: &mut RenderContext<'_>) -> Result<()>;
    fn get_size(&self) -> Result<Size>;
    fn resize(&mut self, size: Size) -> Result<()>;
    fn setup(&self) -> Result<()>;
    fn shutdown(&self) -> Result<()>;
}
