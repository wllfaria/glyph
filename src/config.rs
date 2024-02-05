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
        CONFIG.get_or_init(|| Self {
            line_numbers: LineNumbers::Absolute,
            sidebar_gap: 1,
            sidebar_width: 5,
            empty_line_char: '~',
        })
    }
}
