use std::collections::HashMap;

use crossterm::style::Color;
pub mod loader;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub appearance: Appearance,
    pub statusline: Statusline,
    pub gutter: Style,
    pub tokens: HashMap<String, Style>,
    pub style: Style,
}

#[derive(Debug, Clone)]
pub struct Appearance {
    pub bg: Color,
}

#[derive(Debug, Clone)]
pub struct Statusline {
    pub inner: Style,
}

#[derive(Debug)]
pub struct Gutter {
    pub bg: Color,
    pub fg: Color,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub italic: Option<bool>,
    pub bold: Option<bool>,
    pub underline: Option<bool>,
}

impl Theme {
    pub fn dark() -> anyhow::Result<Self> {
        loader::ThemeLoader::default_dark()
    }

    pub fn light() -> anyhow::Result<Self> {
        loader::ThemeLoader::default_light()
    }
}

impl Default for Theme {
    fn default() -> Self {
        let appearance = Appearance::default();

        Self {
            name: "glyph-dark".to_string(),
            statusline: Statusline::default(),
            gutter: Style::new(appearance.bg),
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
            bg: Color::Rgb { r: 0, g: 0, b: 0 },
        }
    }
}

impl Default for Statusline {
    fn default() -> Self {
        Self {
            inner: Style {
                bg: Some(Color::Rgb { r: 0, g: 0, b: 0 }),
                fg: Some(Color::Rgb {
                    r: 100,
                    g: 100,
                    b: 100,
                }),
                italic: None,
                bold: None,
                underline: None,
            },
        }
    }
}

impl Default for Gutter {
    fn default() -> Self {
        Self {
            bg: Color::Rgb { r: 0, g: 0, b: 0 },
            fg: Color::Rgb {
                r: 100,
                g: 100,
                b: 100,
            },
        }
    }
}