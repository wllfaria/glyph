mod buffer;
mod command;
mod config;
mod editor;
mod events;
mod pane;
mod view;
mod window;

use logger::{self, FileLogger, LogLevel, Logger};

use editor::Editor;

fn main() -> std::io::Result<()> {
    let _ = Logger::new(FileLogger::new("./glyph.log"), LogLevel::None);
    let file_name = std::env::args().nth(1);
    let mut editor = Editor::new(file_name)?;
    editor.start()?;
    Ok(())
}
