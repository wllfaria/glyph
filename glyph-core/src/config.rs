#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum KeyMapPreset {
    Vim,
    VSCode,
}

#[derive(Debug, Default)]
pub struct StatuslineConfig {
    pub mode: StatuslineMode,
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum StatuslineMode {
    #[default]
    Global,
    Local,
}

#[derive(Debug)]
pub struct Config {
    pub keymap_preset: KeyMapPreset,
    pub statusline: StatuslineConfig,
}