#[derive(Debug)]
pub enum KeyMapPreset {
    Vim,
    VSCode,
}

#[derive(Debug)]
pub struct Config {
    pub keymap_preset: KeyMapPreset,
}
