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
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use editor::Editor;
use lsp::LspClient;
use theme::{loader::ThemeLoader, Theme};

fn load_config() -> anyhow::Result<Config> {
    let mut default = Config::default();
    let config_dir = Config::get_path();
    if !config_dir.exists() {
        std::fs::create_dir(&config_dir)?;
    }
    let config_file = config_dir.join("glyph.toml");
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
    let mut editor = Editor::new(&config, &theme, lsp, file_name)?;
    editor.start().await?;
    Ok(())
}
