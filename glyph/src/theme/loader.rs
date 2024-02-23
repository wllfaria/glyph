use std::collections::HashMap;


use crossterm::style::Color;
use serde::Deserialize;

use crate::config::Config;
use crate::theme::{Appearance, Gutter, Statusline, Style, Theme};

#[derive(Deserialize, Debug, Clone)]
struct TokenStyle {
    fg: Option<String>,
    bg: Option<String>,
    italic: Option<bool>,
    bold: Option<bool>,
    underline: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
struct AppearanceStyle {
    fg: String,
    bg: String,
}

#[derive(Deserialize, Debug)]
struct StatuslineStyle {
    inner: TokenStyle,
}

#[derive(Deserialize, Debug)]
struct GutterStyle {
    bg: String,
    fg: String,
}

#[derive(Deserialize, Debug)]
pub struct ThemeLoader {
    name: String,
    appearance: AppearanceStyle,
    statusline: StatuslineStyle,
    gutter: TokenStyle,
    tokens: HashMap<String, TokenStyle>,
}

impl ThemeLoader {
    pub fn default_dark() -> anyhow::Result<Theme> {
        let theme_path = Config::themes_path().join("glyph-dark-default.toml");
        let toml = std::fs::read_to_string(theme_path)?;
        let theme: ThemeLoader = toml::from_str(&toml).unwrap();
        Ok(theme.into())
    }

    pub fn default_light() -> anyhow::Result<Theme> {
        let theme_path = Config::themes_path().join("glyph-light-default.toml");
        let toml = std::fs::read_to_string(theme_path)?;
        let theme: ThemeLoader = toml::from_str(&toml).unwrap();
        Ok(theme.into())
    }
}

pub fn hex_to_rgb(hex: Option<String>) -> Result<Option<Color>, &'static str> {
    match hex {
        Some(hex) => {
            if hex.starts_with('#') && hex.len() == 7 {
                let r = u8::from_str_radix(&hex[1..3], 16).expect("Invalid red component");
                let g = u8::from_str_radix(&hex[3..5], 16).expect("Invalid green component");
                let b = u8::from_str_radix(&hex[5..7], 16).expect("Invalid blue component");

                Ok(Some(Color::Rgb { r, g, b }))
            } else {
                Err("Invalid hex color format")
            }
        }
        None => Ok(None),
    }
}

impl From<ThemeLoader> for Theme {
    fn from(val: ThemeLoader) -> Self {
        let tokens = val.tokens.iter().fold(HashMap::new(), |mut acc, (k, v)| {
            acc.insert(k.clone(), (*v).clone().into());
            acc
        });
        Theme {
            name: val.name,
            statusline: val.statusline.into(),
            gutter: val.gutter.into(),
            tokens,
            style: val.appearance.clone().into(),
            appearance: val.appearance.into(),
        }
    }
}

impl From<AppearanceStyle> for Style {
    fn from(value: AppearanceStyle) -> Self {
        Self {
            fg: hex_to_rgb(Some(value.fg)).unwrap(),
            bg: hex_to_rgb(Some(value.bg)).unwrap(),
            bold: None,
            italic: None,
            underline: None,
        }
    }
}

impl From<AppearanceStyle> for Appearance {
    fn from(val: AppearanceStyle) -> Self {
        Appearance {
            bg: hex_to_rgb(Some(val.bg)).unwrap().unwrap(),
        }
    }
}

impl From<StatuslineStyle> for Statusline {
    fn from(val: StatuslineStyle) -> Self {
        Statusline {
            inner: val.inner.into(),
        }
    }
}

impl From<GutterStyle> for Gutter {
    fn from(val: GutterStyle) -> Self {
        Gutter {
            bg: hex_to_rgb(Some(val.bg)).unwrap().unwrap(),
            fg: hex_to_rgb(Some(val.fg)).unwrap().unwrap(),
        }
    }
}

impl From<TokenStyle> for Style {
    fn from(val: TokenStyle) -> Self {
        Style {
            fg: hex_to_rgb(val.fg).unwrap(),
            bg: hex_to_rgb(val.bg).unwrap(),
            bold: val.bold,
            italic: val.italic,
            underline: val.underline,
        }
    }
}
