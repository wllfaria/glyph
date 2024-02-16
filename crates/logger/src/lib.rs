use chrono::Local;
use std::sync::{Arc, Mutex, OnceLock};

mod file_logger;
mod log_level;
mod logger_error;

#[macro_use]
mod macros;

pub use file_logger::FileLogger;
pub use log_level::LogLevel;
pub use logger_error::LoggerError;

static LOGGER: OnceLock<Logger> = OnceLock::new();

#[allow(dead_code)]
pub struct Logger {
    level: LogLevel,
    writer: Arc<Mutex<dyn std::fmt::Write + Send>>,
}

#[allow(dead_code)]
impl Logger {
    pub fn get<T>(writer: T, level: LogLevel) -> Result<(), LoggerError>
    where
        T: std::fmt::Write + Send + 'static,
    {
        LOGGER
            .set(Logger {
                level,
                writer: Arc::new(Mutex::new(writer)),
            })
            .map_err(|_| LoggerError::AlreadyInitialized("Logger is already initialized"))
    }

    pub fn log(level: LogLevel, args: std::fmt::Arguments) {
        if let Some(logger) = LOGGER.get() {
            match &logger.level {
                LogLevel::None => return,
                logger_level if logger_level > &level => return,
                _ => (),
            }
            let now = Local::now();
            let time = now.format("%Y-%m-%d %H:%M:%S");
            let message = format!("{} [{}] {}", time, level, args);
            let _ = write!(logger.writer.lock().unwrap(), "{}", message);
        }
    }
}
