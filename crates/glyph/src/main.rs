mod buffer;
mod config;
mod editor;
mod events;
mod highlight;
mod lsp;
mod pane;
mod theme;
mod view;
mod viewport;
mod window;

use std::path::{Path, PathBuf};

use config::{Config, EditorBackground};
use logger::{self, FileLogger, LogLevel, Logger};

use editor::Editor;
use lsp::LspClient;
use theme::{loader::ThemeLoader, Theme};

fn load_config() -> anyhow::Result<Config> {
    let mut default = Config::default();
    let config_file = Config::get_path().join("glyph.toml");
    let config_file = Path::new(&config_file);
    match config_file.exists() {
        false => {
            let config_contents = toml::to_string(&default)?;
            std::fs::write(config_file, &config_contents[..])?;

            Ok(default)
        }
        true => {
            let toml = std::fs::read_to_string(config_file)?;
            let config: Config = toml::from_str(&toml)?;
            logger::warn!("{:?}", config.background);
            default.extend(config);
            Ok(default)
        }
    }
}

fn load_theme(
    background: &EditorBackground,
    theme_name: &str,
    themes_dir: PathBuf,
) -> anyhow::Result<Theme> {
    logger::warn!("{background:?}");
    let default = match background {
        EditorBackground::Light => Theme::light()?,
        EditorBackground::Dark => Theme::dark()?,
    };
    if theme_name == "" {
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
    let _ = Logger::get(FileLogger::new("./glyph.log"), LogLevel::Trace);
    let file_name = std::env::args().nth(1);
    let lsp = LspClient::start().await?;
    let config = load_config()?;
    let theme = load_theme(&config.background, &config.theme, Config::themes_path())?;
    let mut editor = Editor::new(&config, &theme, &lsp, file_name)?;
    editor.start().await?;
    Ok(())
}
