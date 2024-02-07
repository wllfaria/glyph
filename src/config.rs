use std::sync;

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
    pub line_numbers: LineNumbers,
    pub sidebar_gap: u16,
    pub sidebar_width: u16,
    pub empty_line_char: char,
}

impl Config {
    pub fn get() -> &'static Self {
        let mut config = Self {
            line_numbers: LineNumbers::Absolute,
            sidebar_gap: 1,
            sidebar_width: 5,
            empty_line_char: '~',
        };

        Config::set_sidebar_width_if_line_numbers_none(&mut config);

        CONFIG.get_or_init(|| config)
    }

    fn set_sidebar_width_if_line_numbers_none(config: &mut Config) {
        match config.line_numbers {
            LineNumbers::None => config.sidebar_width = 1,
            _ => (),
        }
    }
}
