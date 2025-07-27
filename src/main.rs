mod event_loop;
mod renderer;

use std::sync::Arc;

use glyph_core::Glyph;
use glyph_core::config::{Config, KeyMapPreset};
use glyph_core::editing_plugin::EditingPlugin;
use glyph_core::startup_options::StartupOptions;
use glyph_vim::VimEditingPlugin;
use tracing_subscriber::fmt::writer::{BoxMakeWriter, MakeWriterExt};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

fn setup_tracing(verbose: bool) -> eyre::Result<()> {
    let file = std::fs::OpenOptions::new().append(true).open("glyph.log")?;
    // let file = File::open("glyph.log")?;
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

fn editing_plugin_from_config(config: &Config) -> Box<dyn EditingPlugin> {
    match config.keymap_preset {
        KeyMapPreset::Vim => Box::new(VimEditingPlugin::new()),
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
    let editing_plugin = editing_plugin_from_config(&config);

    Glyph::new(
        config,
        event_loop,
        renderer,
        editing_plugin,
        startup_options,
    )?
    .run()?;

    Ok(())
}
