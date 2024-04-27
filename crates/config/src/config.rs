use crate::{default_config::DEFAULT_CONFIG, Config};

use std::path::PathBuf;
use tracing_subscriber::FmtSubscriber;

pub static APP_NAME: &str = "glyph";
pub static THEMES_DIR: &str = "themes";
static CONFIG_FILE: &str = "glyph.toml";

#[cfg(unix)]
static XDG_ENV_VARS: [&str; 2] = ["XDG_CONFIG_HOME", "XDG_DATA_HOME"];

#[cfg(windows)]
static XDG_ENV_VARS: [&str; 2] = ["LOCALAPPDATA", "LOCALAPPDATA"];

#[cfg(unix)]
static XDG_DEFAULTS: [&str; 2] = [".config", ".local/share"];

#[cfg(windows)]
static XDG_DEFAULTS: [&str; 2] = ["AppData\\Local", "AppData\\Local"];

pub fn get_config_dir() -> PathBuf {
    let path = std::env::var(XDG_ENV_VARS[0])
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(XDG_DEFAULTS[0]));

    dirs::home_dir().unwrap_or_default().join(path)
}

#[tracing::instrument]
pub fn load_config() -> Config {
    tracing::info!("loading editor config");
    let config_file = get_config_dir().join(APP_NAME).join(CONFIG_FILE);

    std::fs::read_to_string(config_file)
        .map(|toml| toml::from_str::<Config>(&toml))
        .unwrap_or_else(|_| toml::from_str::<Config>(DEFAULT_CONFIG))
        .unwrap()
}

#[tracing::instrument]
pub fn setup_logger() -> tracing_appender::non_blocking::WorkerGuard {
    tracing::info!("setting up logger");
    let appender = tracing_appender::rolling::never(".", "glyph.log");
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .with_writer(writer)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");
    guard
}
