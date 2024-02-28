use std::collections::HashMap;

use crossterm::style::Color;

use self::loader::hex_to_rgb;
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

        let mut tokens: HashMap<String, Style> = HashMap::new();

        tokens.insert(
            "function".to_string(),
            Style {
                fg: hex_to_rgb(Some("#7daea3".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "function.method".to_string(),
            Style {
                fg: hex_to_rgb(Some("#82aaff".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "function.macro".to_string(),
            Style {
                fg: hex_to_rgb(Some("#ff9e64".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "constant.builtin".to_string(),
            Style {
                fg: hex_to_rgb(Some("#ffcc66".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "constant".to_string(),
            Style {
                fg: hex_to_rgb(Some("#d8a657".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "type".to_string(),
            Style {
                fg: hex_to_rgb(Some("#569CD6".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "type.builtin".to_string(),
            Style {
                fg: hex_to_rgb(Some("#4EC9B0".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "constructor".to_string(),
            Style {
                fg: hex_to_rgb(Some("#B5CEA8".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "property".to_string(),
            Style {
                fg: hex_to_rgb(Some("#CE9178".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "variable.parameter".to_string(),
            Style {
                fg: hex_to_rgb(Some("#9CDCFE".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "variable.builtin".to_string(),
            Style {
                fg: hex_to_rgb(Some("#C586C0".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "label".to_string(),
            Style {
                fg: hex_to_rgb(Some("#D7BA7D".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "comment".to_string(),
            Style {
                fg: hex_to_rgb(Some("#608B4E".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "punctuation.bracket".to_string(),
            Style {
                fg: hex_to_rgb(Some("#D4D4D4".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "punctuation.delimiter".to_string(),
            Style {
                fg: hex_to_rgb(Some("#D4D4D4".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "keyword".to_string(),
            Style {
                fg: hex_to_rgb(Some("#C586C0".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "string".to_string(),
            Style {
                fg: hex_to_rgb(Some("#CE9178".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "escape".to_string(),
            Style {
                fg: hex_to_rgb(Some("#d7ba7d".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "operator".to_string(),
            Style {
                fg: hex_to_rgb(Some("#569CD6".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );
        tokens.insert(
            "attribute".to_string(),
            Style {
                fg: hex_to_rgb(Some("#4EC9B0".to_string())).unwrap(),
                bg: None,
                italic: None,
                underline: None,
                bold: None,
            },
        );

        Self {
            name: "glyph-dark".to_string(),
            statusline: Statusline::default(),
            gutter: Style::new(appearance.bg),
            tokens,
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
