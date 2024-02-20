mod buffer;
mod command;
mod config;
mod editor;
mod events;
mod highlight;
mod lsp;
mod pane;
mod theme;
mod view;
mod viewport;
mod window;

use logger::{self, FileLogger, LogLevel, Logger};

use editor::Editor;
use lsp::LspClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = Logger::get(FileLogger::new("./glyph.log"), LogLevel::Trace);
    let file_name = std::env::args().nth(1);
    let lsp = LspClient::start().await?;
    let mut editor = Editor::new(file_name, &lsp)?;
    editor.start().await?;
    Ok(())
}
