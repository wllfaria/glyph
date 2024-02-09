use crate::LoggerError;

pub trait Writable: Send + Sync + std::fmt::Debug {
    fn write(&self, message: &str) -> Result<(), LoggerError>;
}
