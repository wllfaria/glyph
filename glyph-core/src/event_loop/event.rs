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
        let mut line = String::new();
        let has_any_modifier_but_shift = !(self.modifiers & !KeyModifiers::SHIFT).is_empty();

        if has_any_modifier_but_shift {
            line.push('<');
        }

        if self.modifiers.contains(KeyModifiers::CONTROL) {
            line.push_str("c-");
        }

        if self.modifiers.contains(KeyModifiers::ALT) {
            line.push_str("a-");
        }

        if self.modifiers.contains(KeyModifiers::META) {
            line.push_str("a-");
        }

        let (base_str, always_wrap) = match self.code {
            KeyCode::Char(c) => (c.to_string(), false),

            KeyCode::Backspace => ("bs".into(), true),
            KeyCode::Enter => ("cr".into(), true),
            KeyCode::Left => ("left".into(), true),
            KeyCode::Right => ("right".into(), true),
            KeyCode::Up => ("up".into(), true),
            KeyCode::Down => ("down".into(), true),
            KeyCode::Home => ("home".into(), true),
            KeyCode::End => ("end".into(), true),
            KeyCode::PageUp => ("pgup".into(), true),
            KeyCode::PageDown => ("pgdn".into(), true),
            KeyCode::Tab => ("tab".into(), true),
            KeyCode::BackTab => ("btab".into(), true),
            KeyCode::Delete => ("del".into(), true),
            KeyCode::Insert => ("ins".into(), true),
            KeyCode::F(n) => (format!("f{n}"), true),
            KeyCode::Null => ("null".into(), true),
            KeyCode::Esc => ("esc".into(), true),
            KeyCode::CapsLock => ("caps".into(), true),
            KeyCode::ScrollLock => ("scroll".into(), true),
            KeyCode::NumLock => ("num".into(), true),
            KeyCode::PrintScreen => ("print".into(), true),
            KeyCode::Pause => ("pause".into(), true),
            KeyCode::Menu => ("menu".into(), true),
            KeyCode::KeypadBegin => ("begin".into(), true),

            KeyCode::Media(media) => {
                let label = match media {
                    MediaKeyCode::Play => "play",
                    MediaKeyCode::Pause => "pause",
                    MediaKeyCode::PlayPause => "playpause",
                    MediaKeyCode::Reverse => "reverse",
                    MediaKeyCode::Stop => "stop",
                    MediaKeyCode::FastForward => "ffwd",
                    MediaKeyCode::Rewind => "rewind",
                    MediaKeyCode::TrackNext => "next",
                    MediaKeyCode::TrackPrevious => "prev",
                    MediaKeyCode::Record => "record",
                    MediaKeyCode::LowerVolume => "vol-down",
                    MediaKeyCode::RaiseVolume => "vol-up",
                    MediaKeyCode::MuteVolume => "mute",
                };
                (label.into(), true)
            }

            KeyCode::Modifier(modifier) => {
                let label = match modifier {
                    ModifierKeyCode::LeftShift
                    | ModifierKeyCode::RightShift
                    | ModifierKeyCode::IsoLevel3Shift
                    | ModifierKeyCode::IsoLevel5Shift => "shift",
                    ModifierKeyCode::LeftControl | ModifierKeyCode::RightControl => "ctrl",
                    ModifierKeyCode::LeftAlt | ModifierKeyCode::RightAlt => "alt",
                    ModifierKeyCode::LeftSuper | ModifierKeyCode::RightSuper => "super",
                    ModifierKeyCode::LeftHyper | ModifierKeyCode::RightHyper => "hyper",
                    ModifierKeyCode::LeftMeta | ModifierKeyCode::RightMeta => "meta",
                };
                (label.into(), true)
            }
        };

        line.push_str(&base_str);

        // only close the '>' if a modifier opened it
        if has_any_modifier_but_shift {
            line.push('>');
        } else if always_wrap {
            // If no modifier, but the key is one of the "always wrapped" ones, wrap it
            line = format!("<{line}>");
        }

        write!(f, "{line}")
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