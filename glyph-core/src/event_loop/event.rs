//! Glyph event types are mainly derived from the [`crossterm::event`] module.
//!
//! They are though, defined within glyph to not depend on crossterm types as implementations of
//! the event loop may use different polling mechanisms. Which may produce different types, this is
//! the standard format glyph will use for its events regardless of the underlying library or
//! implementation layer.

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Event {
    Key(KeyEvent),
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    pub kind: KeyEventKind,
}

impl std::fmt::Display for KeyEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.code {
            KeyCode::Char(c) => write!(f, "{c}"),
            KeyCode::Backspace => write!(f, "<bs>"),
            KeyCode::Enter => write!(f, "<cr>"),
            KeyCode::Left => write!(f, "<left>"),
            KeyCode::Right => write!(f, "<right>"),
            KeyCode::Up => write!(f, "<up>"),
            KeyCode::Down => write!(f, "<down>"),
            KeyCode::Home => write!(f, "<home>"),
            KeyCode::End => write!(f, "<end>"),
            KeyCode::PageUp => write!(f, "<pgup>"),
            KeyCode::PageDown => write!(f, "<pgdn>"),
            KeyCode::Tab => write!(f, "<tab>"),
            KeyCode::BackTab => write!(f, "<btab>"),
            KeyCode::Delete => write!(f, "<del>"),
            KeyCode::Insert => write!(f, "<ins>"),
            KeyCode::F(n) => write!(f, "<f{n}>"),
            KeyCode::Null => write!(f, "<null>"),
            KeyCode::Esc => write!(f, "<esc>"),
            KeyCode::CapsLock => write!(f, "<caps>"),
            KeyCode::ScrollLock => write!(f, "<scroll>"),
            KeyCode::NumLock => write!(f, "<num>"),
            KeyCode::PrintScreen => write!(f, "<print>"),
            KeyCode::Pause => write!(f, "<pause>"),
            KeyCode::Menu => write!(f, "<menu>"),
            KeyCode::KeypadBegin => write!(f, "<begin>"),
            KeyCode::Media(media) => match media {
                MediaKeyCode::Play => write!(f, "<play>"),
                MediaKeyCode::Pause => write!(f, "<pause>"),
                MediaKeyCode::PlayPause => write!(f, "<playpause>"),
                MediaKeyCode::Reverse => write!(f, "<reverse>"),
                MediaKeyCode::Stop => write!(f, "<stop>"),
                MediaKeyCode::FastForward => write!(f, "<ffwd>"),
                MediaKeyCode::Rewind => write!(f, "<rewind>"),
                MediaKeyCode::TrackNext => write!(f, "<next>"),
                MediaKeyCode::TrackPrevious => write!(f, "<prev>"),
                MediaKeyCode::Record => write!(f, "<record>"),
                MediaKeyCode::LowerVolume => write!(f, "<vol-down>"),
                MediaKeyCode::RaiseVolume => write!(f, "<vol-up>"),
                MediaKeyCode::MuteVolume => write!(f, "<mute>"),
            },
            KeyCode::Modifier(modifier) => match modifier {
                ModifierKeyCode::LeftShift => write!(f, "<shift>"),
                ModifierKeyCode::LeftControl => write!(f, "<ctrl>"),
                ModifierKeyCode::LeftAlt => write!(f, "<alt>"),
                ModifierKeyCode::LeftSuper => write!(f, "<super>"),
                ModifierKeyCode::LeftHyper => write!(f, "<hyper>"),
                ModifierKeyCode::LeftMeta => write!(f, "<meta>"),
                ModifierKeyCode::RightShift => write!(f, "<shift>"),
                ModifierKeyCode::RightControl => write!(f, "<ctrl>"),
                ModifierKeyCode::RightAlt => write!(f, "<alt>"),
                ModifierKeyCode::RightSuper => write!(f, "<super>"),
                ModifierKeyCode::RightHyper => write!(f, "<hyper>"),
                ModifierKeyCode::RightMeta => write!(f, "<meta>"),
                ModifierKeyCode::IsoLevel3Shift => write!(f, "<shift>"),
                ModifierKeyCode::IsoLevel5Shift => write!(f, "<shift>"),
            },
        }
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Hash)]
pub enum KeyCode {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Null,
    Esc,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
    KeypadBegin,
    Media(MediaKeyCode),
    Modifier(ModifierKeyCode),
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
    pub struct KeyModifiers: u8 {
        const SHIFT = 0b0000_0001;
        const CONTROL = 0b0000_0010;
        const ALT = 0b0000_0100;
        const SUPER = 0b0000_1000;
        const HYPER = 0b0001_0000;
        const META = 0b0010_0000;
        const NONE = 0b0000_0000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyEventKind {
    Press,
    Release,
    Repeat,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Hash)]
pub enum MediaKeyCode {
    Play,
    Pause,
    PlayPause,
    Reverse,
    Stop,
    FastForward,
    Rewind,
    TrackNext,
    TrackPrevious,
    Record,
    LowerVolume,
    RaiseVolume,
    MuteVolume,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ModifierKeyCode {
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    LeftHyper,
    LeftMeta,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    RightHyper,
    RightMeta,
    IsoLevel3Shift,
    IsoLevel5Shift,
}