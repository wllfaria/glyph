mod error;
mod unresolved_config;

use glyph_core::config::Config;

use crate::error::Result;
use crate::unresolved_config::UnresolvedConfig;

static DEFAULT_CONFIG: &str = include_str!("../../config.toml");

pub fn load() -> Result<Config> {
    Ok(default_config())
}

fn default_config() -> Config {
    let unresolved =
        toml::from_str::<UnresolvedConfig>(DEFAULT_CONFIG).expect("default config is valid");

    unresolved.resolve().unwrap()
}