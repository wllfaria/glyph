#[derive(Debug)]
pub enum LoggerError {
    AlreadyInitialized(&'static str),
    FailedToWrite(&'static str),
    FailedToOpen(&'static str),
}
