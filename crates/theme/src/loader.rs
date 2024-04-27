use crate::{
    default_dark::DEFAULT_DARK,
    default_light::DEFAULT_LIGHT,
    theme::{Gutter, StatuslineTheming, Style, Theme},
};
use config::{EditorBackground, APP_NAME, THEMES_DIR};
use crossterm::style::Color;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
struct TokenStyle {
    fg: Option<String>,
    bg: Option<String>,
    italic: Option<bool>,
    bold: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct StatuslineStyle {
    file_name: TokenStyle,
    mode: TokenStyle,
    cursor: TokenStyle,
    appearance: TokenStyle,
}

#[derive(Deserialize, Debug)]
struct GutterStyle {
    bg: String,
    fg: String,
}

#[derive(Deserialize, Debug)]
struct ThemeLoader {
    name: String,
    appearance: TokenStyle,
    statusline: StatuslineStyle,
    float: TokenStyle,
    gutter: TokenStyle,
    tokens: HashMap<String, TokenStyle>,
}

fn default_dark() -> Theme {
    parse_default_dark().into()
}

fn default_light() -> Theme {
    parse_default_light().into()
}

fn parse_default_dark() -> ThemeLoader {
    toml::from_str(DEFAULT_DARK).unwrap()
}

fn parse_default_light() -> ThemeLoader {
    toml::from_str(DEFAULT_LIGHT).unwrap()
}

fn load_custom(theme_name: &str, background: &EditorBackground) -> Theme {
    let theme_path = config::get_config_dir()
        .join(APP_NAME)
        .join(THEMES_DIR)
        .join(theme_name);

    std::fs::read_to_string(theme_path)
        .map(|toml| toml::from_str::<ThemeLoader>(&toml))
        .unwrap_or_else(|_| match background {
            EditorBackground::Dark => Ok(parse_default_dark()),
            EditorBackground::Light => Ok(parse_default_light()),
        })
        .unwrap()
        .into()
}

pub fn load_theme(theme_name: &str, background: &EditorBackground) -> Theme {
    match (theme_name.is_empty(), background) {
        (true, EditorBackground::Dark) => default_dark(),
        (true, EditorBackground::Light) => default_light(),
        (_, _) => load_custom(theme_name, background),
    }
}

fn hex_to_rgb(hex: Option<String>) -> Result<Option<Color>, &'static str> {
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
            float: val.float.into(),
            tokens,
            appearance: val.appearance.into(),
        }
    }
}

impl From<StatuslineStyle> for StatuslineTheming {
    fn from(val: StatuslineStyle) -> Self {
        StatuslineTheming {
            file_name: val.file_name.into(),
            mode: val.mode.into(),
            cursor: val.cursor.into(),
            appearance: val.appearance.into(),
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
        }
    }
}
