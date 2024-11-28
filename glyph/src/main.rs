mod cursor;
mod document;
mod editor;
mod glyph;
mod layers;
mod renderer;
mod tab;
mod tree;
mod window;

use std::io::stdout;

use crossterm::event::EventStream;
use glyph::Glyph;
use glyph_term::backend::CrosstermBackend;
use tracing_appender::non_blocking::WorkerGuard;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let guard = setup_logger();

    let backend = CrosstermBackend::new(stdout());

    let mut glyph = Glyph::new(backend);
    glyph.run(&mut EventStream::new()).await?;

    _ = guard;

    Ok(())
}

fn setup_logger() -> WorkerGuard {
    if std::fs::exists("./glyph.log").unwrap_or_default() {
        _ = std::fs::remove_file("./glyph.log");
    }
    let file_appender = tracing_appender::rolling::never(".", "glyph.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_level(true)
        .with_ansi(false)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global default subscriber");

    _guard
}
