mod buffer;
mod config;
mod cursor;
mod editor;
mod events;
mod highlight;
mod lsp;
mod pane;
mod theme;
mod tui;
mod view;
mod viewport;
mod window;

use std::{
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

use config::{Config, EditorBackground};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use editor::Editor;
use lsp::LspClient;
use theme::{loader::ThemeLoader, Theme};

fn load_config() -> anyhow::Result<Config> {
    let config_dir = Config::get_path();
    let config_file = config_dir.join("glyph.toml");
    let config_file = Path::new(&config_file);
    // TODO: in the future, the initial config should be installed automatically
    if !config_file.exists() {
        tracing::error!("loaded failed");
        return Err(anyhow::Error::new(Error::new(
            ErrorKind::NotFound,
            "config file not found. please refer to the configuration section of the readme",
        )));
    }
    let toml = std::fs::read_to_string(config_file)?;
    tracing::error!("loaded config");
    let config: Config = toml::from_str(&toml)?;
    Ok(config)
}

fn load_theme(
    background: &EditorBackground,
    theme_name: &str,
    themes_dir: PathBuf,
) -> anyhow::Result<Theme> {
    if !themes_dir.exists() {
        std::fs::create_dir(&themes_dir)?;
        // TODO: install themes when first loading
    }
    let default = match background {
        EditorBackground::Light => Theme::light()?,
        EditorBackground::Dark => Theme::dark()?,
    };
    if theme_name.is_empty() {
        return Ok(default);
    }
    let theme_path = themes_dir.join(theme_name);
    match theme_path.exists() {
        false => Ok(default),
        true => {
            let toml = std::fs::read_to_string(theme_path)?;
            let theme: ThemeLoader = toml::from_str(&toml)?;
            Ok(theme.into())
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let appender = tracing_appender::rolling::never(".", "glyph.log");
    let (writer, _guard) = tracing_appender::non_blocking(appender);
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(writer)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    let file_name = std::env::args().nth(1);
    let lsp = LspClient::start().await?;
    let config = load_config()?;
    let theme = load_theme(&config.background, &config.theme, Config::themes_path())?;
    Editor::new(&config, &theme, lsp, file_name).await?;
    Ok(())
}
