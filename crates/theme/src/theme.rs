use crate::loader::ThemeLoader;
use crossterm::style::Color;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub appearance: Style,
    pub statusline: StatuslineTheming,
    pub float: Style,
    pub gutter: Style,
    pub tokens: HashMap<String, Style>,
}

#[cfg(test)]
impl Default for Theme {
    fn default() -> Self {
        Self {
            name: String::from("default"),
            appearance: Style::default(),
            statusline: StatuslineTheming::default(),
            float: Style::default(),
            gutter: Style::default(),
            tokens: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StatuslineTheming {
    pub file_name: Style,
    pub mode: Style,
    pub cursor: Style,
    pub appearance: Style,
}

#[derive(Debug)]
pub struct Gutter {
    pub bg: Color,
    pub fg: Color,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub italic: Option<bool>,
    pub bold: Option<bool>,
}

impl Theme {
    pub fn dark() -> anyhow::Result<Self> {
        ThemeLoader::default_dark()
    }

    pub fn light() -> anyhow::Result<Self> {
        ThemeLoader::default_light()
    }
}
