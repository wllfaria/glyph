use glyph_core::config::{Config, KeyMapPreset};
use serde::Deserialize;

use crate::error::{ConfigError, Result};

#[derive(Deserialize)]
pub struct UnresolvedConfig {
    keymap_preset: Option<String>,
}

impl UnresolvedConfig {
    pub fn resolve(self) -> Result<Config> {
        let keymap_preset = self
            .keymap_preset
            .map(parse_keymap_preset)
            .transpose()?
            .unwrap_or(KeyMapPreset::Vim);

        Ok(Config { keymap_preset })
    }
}

fn parse_keymap_preset<S: AsRef<str>>(s: S) -> Result<KeyMapPreset> {
    match s.as_ref() {
        "vim" => Ok(KeyMapPreset::Vim),
        "vscode" => Ok(KeyMapPreset::VSCode),
        _ => Err(ConfigError::InvalidOption),
    }
}
