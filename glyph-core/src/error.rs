use crate::event_loop::error::EventLoopError;
use crate::renderer::error::RendererError;

pub type Result<T, E = GlyphError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum GlyphError {
    #[error("Failed to canonicalize path: {0}")]
    FailedToCanonicalizePath(#[from] std::io::Error),

    #[error(transparent)]
    RendererError(#[from] RendererError),

    #[error(transparent)]
    EventLoopError(#[from] EventLoopError),
}
