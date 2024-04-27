mod cursor;
mod editor;
mod events;
mod frame;
mod highlight;
mod tui;

use anyhow::Context;
use editor::Editor;
use lsp::LspClient;

#[tokio::main]
#[tracing::instrument(err)]
async fn main() -> anyhow::Result<()> {
    let _guard = config::setup_logger();

    let lsp = LspClient::start()
        .await
        .context("failed to initialize LSP server")?;

    let config = config::load_config();
    let theme = theme::load_theme(&config.theme, &config.background);

    let file_name = std::env::args().nth(1);
    Editor::new(&config, &theme, lsp, file_name)
        .await
        .context("failed to start editor")?;

    Ok(())
}
