use chrono::Local;
use std::sync::{Arc, OnceLock};

mod file_logger;
mod log_level;
mod logger_error;
#[macro_use]
mod macros;
mod stdout_logger;
mod writable;

use writable::Writable;

pub use file_logger::FileLogger;
pub use log_level::LogLevel;
pub use logger_error::LoggerError;
pub use stdout_logger::StdoutLogger;

static LOGGER: OnceLock<Logger> = OnceLock::new();

#[derive(Debug)]
#[allow(dead_code)]
pub struct Logger {
    writer: Arc<dyn Writable>,
}

#[allow(dead_code)]
impl Logger {
    pub fn new<T>(writer: T) -> Result<(), LoggerError>
    where
        T: Writable + 'static,
    {
        LOGGER
            .set(Logger {
                writer: Arc::new(writer),
            })
            .map_err(|_| LoggerError::AlreadyInitialized("Logger is already initialized"))
    }

    pub fn log(level: LogLevel, message: &str) {
        if let Some(logger) = LOGGER.get() {
            let now = Local::now();
            let time = now.format("%Y-%m-%d %H:%M:%S");
            let message = format!("{} [{}] {}", time, level, message);
            let _ = logger.writer.write(&message);
        }
    }
}
