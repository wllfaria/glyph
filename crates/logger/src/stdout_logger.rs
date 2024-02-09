use crate::{logger_error::LoggerError, writable::Writable};

#[derive(Debug)]
pub struct StdoutLogger {}

impl Writable for StdoutLogger {
    fn write(&self, message: &str) -> Result<(), LoggerError> {
        println!("{}", message);
        Ok(())
    }
}

impl Default for StdoutLogger {
    fn default() -> Self {
        Self {}
    }
}
