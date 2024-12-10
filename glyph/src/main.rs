mod glyph;

use std::io::stdout;

use crossterm::event::EventStream;
use glyph::Glyph;
use glyph_config::dirs::DIRS;
use glyph_term::backend::CrosstermBackend;
use tokio::sync::mpsc::unbounded_channel;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    glyph_config::dirs::setup_dirs();
    let _guard = setup_logger();

    let (runtime_sender, mut runtime_receiver) = unbounded_channel();
    let runtime = glyph_runtime::setup_lua_runtime(DIRS.get().unwrap().config(), runtime_sender.clone())?;

    let mut config = glyph_config::Config::load(&runtime, runtime_sender.clone(), &mut runtime_receiver)?;

    let backend = CrosstermBackend::new(stdout());

    let mut glyph = Glyph::new(backend, runtime, runtime_sender, runtime_receiver, &mut config)?;
    glyph.run(&mut EventStream::new()).await?;

    Ok(())
}

fn setup_logger() -> WorkerGuard {
    let file_appender = tracing_appender::rolling::never(DIRS.get().unwrap().data(), "glyph.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_level(true)
        .with_max_level(LevelFilter::DEBUG)
        .with_ansi(false)
        .init();

    guard
}
