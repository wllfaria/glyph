mod event_loop;
mod renderer;

use glyph_core::Glyph;
use glyph_core::startup_options::StartupOptions;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let startup_options = StartupOptions::from_args();
    let event_loop = event_loop::CrosstermEventLoop;
    let renderer = renderer::CrosstermRenderer::new()?;

    Glyph::new(event_loop, renderer, startup_options)?.run()?;

    Ok(())
}
