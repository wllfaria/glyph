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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = CrosstermBackend::new(stdout());

    let mut glyph = Glyph::new(backend);
    glyph.run(&mut EventStream::new()).await?;

    Ok(())
}
