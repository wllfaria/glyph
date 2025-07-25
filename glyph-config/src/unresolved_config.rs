use glyph_core::config::{Config, KeyMapPreset, StatuslineConfig, StatuslineMode};
use serde::Deserialize;

use crate::error::{ConfigError, Result};

#[derive(Deserialize)]
pub struct UnresolvedConfig {
    keymap_preset: Option<String>,
    statusline: Option<UnresolvedStatuslineConfig>,
}

#[derive(Deserialize)]
pub struct UnresolvedStatuslineConfig {
    mode: String,
}

impl UnresolvedConfig {
    pub fn resolve(self) -> Result<Config> {
        let keymap_preset = self
            .keymap_preset
            .map(parse_keymap_preset)
            .transpose()?
            .unwrap_or(KeyMapPreset::Vim);

        let statusline = self
            .statusline
            .map(parse_statusline)
            .transpose()?
            .unwrap_or(StatuslineConfig::default());

        Ok(Config {
            keymap_preset,
            statusline,
        })
    }
}

fn parse_keymap_preset<S: AsRef<str>>(s: S) -> Result<KeyMapPreset> {
    match s.as_ref() {
        "vim" => Ok(KeyMapPreset::Vim),
        "vscode" => Ok(KeyMapPreset::VSCode),
        _ => Err(ConfigError::InvalidOption),
    }
}

fn parse_statusline(unresolved: UnresolvedStatuslineConfig) -> Result<StatuslineConfig> {
    let mode = match unresolved.mode.to_lowercase().as_str() {
        "local" => StatuslineMode::Local,
        "global" => StatuslineMode::Global,
        _ => return Err(ConfigError::InvalidOption),
    };

    Ok(StatuslineConfig { mode })
}