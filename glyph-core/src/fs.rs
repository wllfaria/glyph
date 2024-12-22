use std::fmt::Display;
use std::io::Write;
use std::path::Path;

pub fn write_file<P: AsRef<Path>, T: Display>(path: P, text: T) -> Result<usize, std::io::Error> {
    let path = path.as_ref();

    let mut handle = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    let bytes_written = handle.write(text.to_string().as_bytes())?;

    Ok(bytes_written)
}
