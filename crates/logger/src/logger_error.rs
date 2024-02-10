#[derive(Debug)]
pub enum LoggerError {
    AlreadyInitialized(&'static str),
    FailedToWrite(&'static str),
    FailedToOpen(&'static str),
}

impl std::fmt::Display for LoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            LoggerError::FailedToWrite(ref message) => {
                write!(f, "Failed to write to log file: {}", message)
            }
            LoggerError::FailedToOpen(ref message) => {
                write!(f, "Failed to open log file: {}", message)
            }
            LoggerError::AlreadyInitialized(ref message) => {
                write!(f, "{}", message)
            }
        }
    }
}

impl std::error::Error for LoggerError {}
