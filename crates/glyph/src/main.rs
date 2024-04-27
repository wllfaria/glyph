mod cursor;
mod editor;
mod events;
mod frame;
mod highlight;
mod tui;

use config::EditorBackground;
use editor::Editor;
use lsp::LspClient;
use theme::Theme;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn load_theme(background: &EditorBackground) -> anyhow::Result<Theme> {
    let default = match background {
        EditorBackground::Light => Theme::light()?,
        EditorBackground::Dark => Theme::dark()?,
    };
    Ok(default)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let appender = tracing_appender::rolling::never(".", "glyph.log");
    let (writer, _guard) = tracing_appender::non_blocking(appender);
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_ansi(false)
        .with_writer(writer)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    let file_name = std::env::args().nth(1);

    let lsp = LspClient::start().await?;
    let config = config::load_config();
    let theme = load_theme(&config.background)?;

    Editor::new(&config, &theme, lsp, file_name).await?;
    Ok(())
}
