use std::{path::PathBuf, sync};

static CONFIG: sync::OnceLock<Config> = sync::OnceLock::new();

#[derive(Debug)]
pub enum LineNumbers {
    Absolute,
    Relative,
    RelativeNumbered,
    None,
}

#[derive(Debug)]
pub struct Config {
    pub theme_name: String,
    pub line_numbers: LineNumbers,
    pub gutter_width: u16,
    pub empty_line_char: char,
}

impl Config {
    pub fn get() -> &'static Self {
        let mut config = Self {
            theme_name: "theme".to_string(),
            line_numbers: LineNumbers::Absolute,
            gutter_width: 6,
            empty_line_char: '~',
        };

        Config::set_sidebar_width_if_line_numbers_none(&mut config);

        CONFIG.get_or_init(|| config)
    }

    pub fn config_dir() -> PathBuf {
        dirs::home_dir().unwrap().join(".config/glyph")
    }

    pub fn themes_dir() -> PathBuf {
        Config::config_dir().join("themes")
    }

    fn set_sidebar_width_if_line_numbers_none(config: &mut Config) {
        if let LineNumbers::None = config.line_numbers {
            config.gutter_width = 1;
        }
    }
}
