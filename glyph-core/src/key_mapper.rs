use std::fmt::Debug;

use crate::event_loop::event::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EditorMode {
    Vim(VimMode),
    VSCode,
    Emacs,
}

impl EditorMode {
    pub fn expect_vim(self) -> VimMode {
        match self {
            Self::Vim(vim_mode) => vim_mode,
            Self::VSCode => panic!("invalid editor mode, expected vim mode, found VSCode"),
            Self::Emacs => panic!("invalid editor mode, expected vim mode, found Emacs"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VimMode {
    #[default]
    Normal,
    Insert,
    Visual,
}

impl std::fmt::Display for VimMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "normal"),
            Self::Insert => write!(f, "insert"),
            Self::Visual => write!(f, "visual"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Command {
    MoveCursorLeft,
    MoveCursorDown,
    MoveCursorUp,
    MoveCursorRight,
    MoveCursorLineStart,
    MoveCursorLineEnd,
    MoveToMatchingPair,
    DeleteWholeLine,
    MoveToTop,
    MoveToBottom,
    MoveToFirstNonSpace,
    MoveToLastNonSpace,
    MoveToNextParagraph,
    MoveToPrevParagraph,
    DeletePrevChar,
    DeleteCurrChar,
    MoveToNextWord,
    TypeChar(char),
    PageUp,
    PageDown,
    Quit,
}

pub struct ResolvedKeymap {
    pub commands: Vec<Command>,
    pub mode: Option<EditorMode>,
}

pub trait Keymapper: Debug {
    fn parse_event(&mut self, event: Option<Event>) -> Option<ResolvedKeymap>;
    fn mode(&self) -> EditorMode;
}