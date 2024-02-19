mod buffer;
mod command;
mod config;
mod editor;
mod events;
mod highlight;
mod pane;
mod theme;
mod view;
mod viewport;
mod window;

use logger::{self, FileLogger, LogLevel, Logger};

use editor::Editor;

fn main() -> std::io::Result<()> {
    let _ = Logger::get(FileLogger::new("./glyph.log"), LogLevel::Trace);
    let file_name = std::env::args().nth(1);
    let mut editor = Editor::new(file_name)?;
    editor.start()?;
    Ok(())
}
