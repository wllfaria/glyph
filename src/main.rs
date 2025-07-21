mod event_loop;
mod renderer;

use glyph_core::Glyph;
use glyph_core::config::{Config, KeyMapPreset};
use glyph_core::key_mapper::KeyMapperKind;
use glyph_core::startup_options::StartupOptions;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let startup_options = StartupOptions::from_args();
    let config = glyph_config::load()?;
    let event_loop = event_loop::CrosstermEventLoop;
    let renderer = renderer::CrosstermRenderer::new()?;
    let key_mapper = key_mapper_from_config(&config);

    Glyph::new(config, event_loop, renderer, key_mapper, startup_options)?.run()?;

    Ok(())
}

fn key_mapper_from_config(config: &Config) -> KeyMapperKind {
    match config.keymap_preset {
        KeyMapPreset::Vim => glyph_core::key_mapper::VimKeyMapper::new().into(),
        KeyMapPreset::VSCode => glyph_core::key_mapper::VSCodeKeyMapper::new().into(),
    }
}
