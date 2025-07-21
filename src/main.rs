mod event_loop;
mod renderer;

use std::fs::File;

use glyph_core::Glyph;
use glyph_core::config::{Config, KeyMapPreset};
use glyph_core::key_mapper::KeymapperKind;
use glyph_core::startup_options::StartupOptions;
use tracing_subscriber::fmt::writer::{BoxMakeWriter, MakeWriterExt};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

fn setup_tracing(verbose: bool) -> eyre::Result<()> {
    let file = File::create("glyph.log")?;
    let writer = BoxMakeWriter::new(file);

    #[cfg(debug_assertions)]
    let mut env_filter = EnvFilter::from("debug");

    #[cfg(not(debug_assertions))]
    let mut env_filter = EnvFilter::from_default_env();

    if verbose {
        env_filter = EnvFilter::from("trace");
    }

    let subscriber = FmtSubscriber::builder()
        .with_writer(writer.with_max_level(tracing::Level::TRACE))
        .with_env_filter(env_filter)
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

fn key_mapper_from_config(config: &Config) -> KeymapperKind {
    match config.keymap_preset {
        KeyMapPreset::Vim => glyph_core::key_mapper::VimKeymapper::new().into(),
        KeyMapPreset::VSCode => glyph_core::key_mapper::VSCodeKeymapper::new().into(),
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let startup_options = StartupOptions::from_args();

    setup_tracing(startup_options.verbose)?;

    let config = glyph_config::load()?;
    let event_loop = event_loop::CrosstermEventLoop;
    let renderer = renderer::CrosstermRenderer::new()?;
    let key_mapper = key_mapper_from_config(&config);

    Glyph::new(config, event_loop, renderer, key_mapper, startup_options)?.run()?;

    Ok(())
}
