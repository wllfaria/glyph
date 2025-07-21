pub type Result<T, E = ConfigError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("invalid option")]
    InvalidOption,
}
