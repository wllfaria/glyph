mod vim_key_mapper;
mod vsc_key_mapper;

pub use vim_key_mapper::VimKeyMapper;
pub use vsc_key_mapper::VSCodeKeyMapper;

use crate::event_loop::event::Event;

pub enum UiCommand {}

pub enum Command {
    UiCommand(UiCommand),
}

#[derive(Debug)]
pub enum KeyMapperKind {
    Vim(VimKeyMapper),
    VSCode(VSCodeKeyMapper),
}

impl KeyMapper for KeyMapperKind {
    fn parse_event(&mut self, event: Option<Event>) -> Option<Command> {
        match self {
            Self::Vim(vim) => vim.parse_event(event),
            Self::VSCode(vsc) => vsc.parse_event(event),
        }
    }
}

impl From<VimKeyMapper> for KeyMapperKind {
    fn from(k: VimKeyMapper) -> Self {
        Self::Vim(k)
    }
}

impl From<VSCodeKeyMapper> for KeyMapperKind {
    fn from(k: VSCodeKeyMapper) -> Self {
        Self::VSCode(k)
    }
}

pub trait KeyMapper {
    fn parse_event(&mut self, event: Option<Event>) -> Option<Command>;
}
