use std::{collections::HashMap, sync::OnceLock};

use crossterm::style::Color;

mod loader;

pub static THEME: OnceLock<Theme> = OnceLock::new();

#[derive(Debug)]
pub struct Theme {
    pub name: String,
    pub appearance: Appearance,
    pub statusline: Statusline,
    pub gutter: Gutter,
    pub tokens: HashMap<String, Style>,
    pub style: Style,
}

#[derive(Debug)]
pub struct Appearance {
    pub bg: Color,
}

#[derive(Debug)]
pub struct Statusline {
    pub bg: Color,
}

#[derive(Debug)]
pub struct Gutter {
    pub bg: Color,
}

#[derive(Debug)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub italic: Option<bool>,
    pub bold: Option<bool>,
    pub underline: Option<bool>,
}

impl Theme {
    pub fn get() -> &'static Self {
        THEME.get_or_init(|| match loader::ThemeLoader::parse_theme() {
            Ok(theme) => {
                logger::debug!("{theme:?}");
                theme
            }
            Err(_) => Self::default(),
        })
    }
}

impl Default for Theme {
    fn default() -> Self {
        let appearance = Appearance::default();
        Self {
            name: "glyph-dark".to_string(),
            statusline: Statusline::default(),
            gutter: Gutter::default(),
            tokens: HashMap::new(),
            style: Style::new(appearance.bg),
            appearance,
        }
    }
}

impl Style {
    pub fn new(bg: Color) -> Self {
        Self {
            fg: Some(Color::Rgb {
                r: 255,
                g: 255,
                b: 255,
            }),
            bg: Some(bg),
            bold: None,
            italic: None,
            underline: None,
        }
    }
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            bg: Color::Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
        }
    }
}

impl Default for Statusline {
    fn default() -> Self {
        Self {
            bg: Color::Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
        }
    }
}

impl Default for Gutter {
    fn default() -> Self {
        Self {
            bg: Color::Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
        }
    }
}
