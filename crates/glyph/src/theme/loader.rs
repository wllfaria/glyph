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

#[derive(Deserialize, Debug)]
struct AppearanceStyle {
    bg: String,
}

#[derive(Deserialize, Debug)]
struct StatuslineStyle {
    bg: String,
}

#[derive(Deserialize, Debug)]
struct GutterStyle {
    bg: String,
}

#[derive(Deserialize, Debug)]
pub struct ThemeLoader {
    name: String,
    appearance: AppearanceStyle,
    statusline: StatuslineStyle,
    gutter: GutterStyle,
    tokens: HashMap<String, TokenStyle>,
}

impl ThemeLoader {
    pub fn parse_theme() -> std::io::Result<Theme> {
        let mut theme_name = Config::get().theme_name.clone();
        theme_name.push_str(".toml");
        let theme_path = Config::themes_dir().join(theme_name);
        if !theme_path.exists() {
            println!("no theme");
        }
        let toml = std::fs::read_to_string(theme_path)?;
        let theme: ThemeLoader = toml::from_str(&toml).unwrap();
        Ok(theme.into())
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
}

impl From<ThemeLoader> for Theme {
    fn from(val: ThemeLoader) -> Self {
        let tokens = val.tokens.iter().fold(HashMap::new(), |mut acc, (k, v)| {
            acc.insert(k.clone(), (*v).clone().into());
            acc
        });
        let appearance: Appearance = val.appearance.into();
        Theme {
            name: val.name,
            statusline: val.statusline.into(),
            gutter: val.gutter.into(),
            tokens,
            style: Style::new(appearance.bg),
            appearance,
        }
    }
}

impl From<AppearanceStyle> for Appearance {
    fn from(val: AppearanceStyle) -> Self {
        Appearance {
            bg: ThemeLoader::hex_to_rgb(Some(val.bg)).unwrap().unwrap(),
        }
    }
}

impl From<StatuslineStyle> for Statusline {
    fn from(val: StatuslineStyle) -> Self {
        Statusline {
            bg: ThemeLoader::hex_to_rgb(Some(val.bg)).unwrap().unwrap(),
        }
    }
}

impl From<GutterStyle> for Gutter {
    fn from(val: GutterStyle) -> Self {
        Gutter {
            bg: ThemeLoader::hex_to_rgb(Some(val.bg)).unwrap().unwrap(),
        }
    }
}

impl From<TokenStyle> for Style {
    fn from(val: TokenStyle) -> Self {
        Style {
            fg: ThemeLoader::hex_to_rgb(val.fg).unwrap(),
            bg: ThemeLoader::hex_to_rgb(val.bg).unwrap(),
            bold: val.bold,
            italic: val.italic,
            underline: val.underline,
        }
    }
}
