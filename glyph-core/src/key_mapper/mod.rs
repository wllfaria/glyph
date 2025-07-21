mod vim_key_mapper;
mod vsc_key_mapper;

pub use vim_key_mapper::VimKeymapper;
pub use vsc_key_mapper::VSCodeKeymapper;

use crate::event_loop::event::Event;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Command {
    MoveCursorLeft,
    MoveCursorDown,
    MoveCursorUp,
    MoveCursorRight,
}

#[derive(Debug)]
pub enum KeymapperKind {
    Vim(VimKeymapper),
    VSCode(VSCodeKeymapper),
}

impl Keymapper for KeymapperKind {
    fn parse_event(&mut self, event: Option<Event>) -> Option<Vec<Command>> {
        match self {
            Self::Vim(vim) => vim.parse_event(event),
            Self::VSCode(vsc) => vsc.parse_event(event),
        }
    }
}

impl From<VimKeymapper> for KeymapperKind {
    fn from(k: VimKeymapper) -> Self {
        Self::Vim(k)
    }
}

impl From<VSCodeKeymapper> for KeymapperKind {
    fn from(k: VSCodeKeymapper) -> Self {
        Self::VSCode(k)
    }
}

pub trait Keymapper {
    fn parse_event(&mut self, event: Option<Event>) -> Option<Vec<Command>>;
}
