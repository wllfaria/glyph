mod editor;
mod glyph;

use std::io::stdout;

use crossterm::event::EventStream;
use glyph::Glyph;
use glyph_tui::backend::CrosstermBackend;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = CrosstermBackend::new(stdout());
    let mut glyph = Glyph::new(backend);
    glyph.run(&mut EventStream::new()).await?;
    Ok(())
}
