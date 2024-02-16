use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

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

impl std::fmt::Write for FileLogger {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|_| std::fmt::Error)?;
        writeln!(file, "{}", s).map_err(|_| std::fmt::Error)
    }
}
