use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::logger_error::LoggerError;
use crate::writable::Writable;

#[derive(Debug)]
pub struct FileLogger {
    path: PathBuf,
}

impl FileLogger {
    pub fn new<T>(path: T) -> Self
    where
        T: AsRef<Path>,
    {
        Self {
            path: path.as_ref().to_owned(),
        }
    }
}

impl Writable for FileLogger {
    fn write(&self, message: &str) -> Result<(), LoggerError> {
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            Ok(mut file) => writeln!(file, "{}", message)
                .map_err(|_| LoggerError::FailedToWrite("Failed to write to log file")),
            Err(_) => Err(LoggerError::FailedToOpen("Failed to open log file")),
        }
    }
}
