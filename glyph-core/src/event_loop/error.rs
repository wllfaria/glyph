pub type Result<T, E = EventLoopError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum EventLoopError {
    #[error("Failed to pool for events")]
    FailedToPool,
    #[error("Failed to read event: {0}")]
    FailedToReadEvent(#[from] std::io::Error),
}
