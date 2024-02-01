use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug)]
pub struct Config {
    pub line_numbers: bool,
    pub sidebar_gap: u16,
    pub sidebar_width: u16,
}

impl Config {
    pub fn get() -> &'static Self {
        CONFIG.get_or_init(|| Self {
            line_numbers: false,
            sidebar_gap: 1,
            sidebar_width: 5,
        })
    }
}
