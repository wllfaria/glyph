#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

#[derive(Debug)]
pub struct Editor {
    pub mode: Mode,
    pub should_close: bool,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            should_close: false,
            mode: Mode::Normal,
        }
    }
}
