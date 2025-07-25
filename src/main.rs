mod event_loop;
mod renderer;

use std::fs::File;
use std::sync::Arc;

use glyph_core::Glyph;
use glyph_core::command_handler::CommandHandler;
use glyph_core::config::{Config, KeyMapPreset};
use glyph_core::key_mapper::Keymapper;
use glyph_core::startup_options::StartupOptions;
use glyph_core::status_provider::StatuslineProvider;
use glyph_vim::{VimBufferCommandHandler, VimKeymapper, VimStatusline};
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

fn key_mapper_from_config(config: &Config) -> Box<dyn Keymapper> {
    match config.keymap_preset {
        KeyMapPreset::Vim => Box::new(VimKeymapper::new()),
        KeyMapPreset::VSCode => todo!(),
    }
}

fn handler_from_config(config: &Config) -> Box<dyn CommandHandler> {
    match config.keymap_preset {
        KeyMapPreset::Vim => Box::new(VimBufferCommandHandler),
        KeyMapPreset::VSCode => todo!(),
    }
}

fn statusline_provider_from_config(config: &Config) -> Box<dyn StatuslineProvider> {
    match config.keymap_preset {
        KeyMapPreset::Vim => Box::new(VimStatusline),
        KeyMapPreset::VSCode => todo!(),
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let startup_options = StartupOptions::from_args();

    setup_tracing(startup_options.verbose)?;

    let config = Arc::new(glyph_config::load()?);
    let event_loop = event_loop::CrosstermEventLoop;
    let renderer = renderer::CrosstermRenderer::new(config.clone())?;
    let key_mapper = key_mapper_from_config(&config);
    let command_handler = handler_from_config(&config);
    let statusline_provider = statusline_provider_from_config(&config);

    Glyph::new(
        config,
        event_loop,
        renderer,
        key_mapper,
        command_handler,
        statusline_provider,
        startup_options,
    )?
    .run()?;

    Ok(())
}
