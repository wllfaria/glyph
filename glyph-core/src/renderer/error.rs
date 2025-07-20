pub type Result<T, E = RendererError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Failed to get editor size")]
    FailedToGetEditorSize,

    #[error("Failed to setup renderer")]
    FailedToSetupRenderer,

    #[error("Failed to shutdown renderer")]
    FailedToShutdownRenderer,
}
