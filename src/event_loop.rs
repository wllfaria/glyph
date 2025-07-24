use crossterm::event::{
    Event as CTEvent, KeyCode as CTKeyCode, KeyEvent as CTKeyEvent, KeyEventKind as CTKeyEventKind,
    KeyModifiers as CTKeyModifiers, MediaKeyCode as CTMediaKeyCode,
    ModifierKeyCode as CTModifierKeyCode, poll,
};
use glyph_core::event_loop::EventLoop;
use glyph_core::event_loop::error::{EventLoopError, Result};
use glyph_core::event_loop::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MediaKeyCode, ModifierKeyCode,
};

trait IntoEvent {
    type Target;
    fn into_event(self) -> Self::Target;
}

#[derive(Debug)]
pub struct CrosstermEventLoop;

impl EventLoop for CrosstermEventLoop {
    fn maybe_event(&self) -> Result<Option<Event>> {
        let poll_timeout = std::time::Duration::from_millis(30);

        let Ok(has_event) = poll(poll_timeout) else {
            return Err(EventLoopError::FailedToPool);
        };

        if !has_event {
            return Ok(None);
        }

        match crossterm::event::read() {
            Ok(event) => Ok(Some(event.into_event())),
            Err(e) => Err(EventLoopError::FailedToReadEvent(e)),
        }
    }
}

impl IntoEvent for CTEvent {
    type Target = Event;

    fn into_event(self) -> Self::Target {
        match self {
            CTEvent::Key(event) => Event::Key(event.into_event()),
            _ => unimplemented!(),
        }
    }
}

impl IntoEvent for CTKeyEvent {
    type Target = KeyEvent;

    fn into_event(self) -> Self::Target {
        KeyEvent {
            code: self.code.into_event(),
            modifiers: self.modifiers.into_event(),
            kind: self.kind.into_event(),
        }
    }
}

impl IntoEvent for CTKeyCode {
    type Target = KeyCode;

    fn into_event(self) -> Self::Target {
        match self {
            CTKeyCode::Backspace => KeyCode::Backspace,
            CTKeyCode::Enter => KeyCode::Enter,
            CTKeyCode::Left => KeyCode::Left,
            CTKeyCode::Right => KeyCode::Right,
            CTKeyCode::Up => KeyCode::Up,
            CTKeyCode::Down => KeyCode::Down,
            CTKeyCode::Home => KeyCode::Home,
            CTKeyCode::End => KeyCode::End,
            CTKeyCode::PageUp => KeyCode::PageUp,
            CTKeyCode::PageDown => KeyCode::PageDown,
            CTKeyCode::Tab => KeyCode::Tab,
            CTKeyCode::BackTab => KeyCode::BackTab,
            CTKeyCode::Delete => KeyCode::Delete,
            CTKeyCode::Insert => KeyCode::Insert,
            CTKeyCode::F(n) => KeyCode::F(n),
            CTKeyCode::Char(c) => KeyCode::Char(c),
            CTKeyCode::Null => KeyCode::Null,
            CTKeyCode::Esc => KeyCode::Esc,
            CTKeyCode::CapsLock => KeyCode::CapsLock,
            CTKeyCode::ScrollLock => KeyCode::ScrollLock,
            CTKeyCode::NumLock => KeyCode::NumLock,
            CTKeyCode::PrintScreen => KeyCode::PrintScreen,
            CTKeyCode::Pause => KeyCode::Pause,
            CTKeyCode::Menu => KeyCode::Menu,
            CTKeyCode::KeypadBegin => KeyCode::KeypadBegin,
            CTKeyCode::Media(code) => KeyCode::Media(code.into_event()),
            CTKeyCode::Modifier(code) => KeyCode::Modifier(code.into_event()),
        }
    }
}

impl IntoEvent for CTKeyModifiers {
    type Target = KeyModifiers;

    fn into_event(self) -> Self::Target {
        KeyModifiers::from_bits_retain(self.bits())
    }
}

impl IntoEvent for CTKeyEventKind {
    type Target = KeyEventKind;

    fn into_event(self) -> Self::Target {
        match self {
            CTKeyEventKind::Press => KeyEventKind::Press,
            CTKeyEventKind::Release => KeyEventKind::Release,
            CTKeyEventKind::Repeat => KeyEventKind::Repeat,
        }
    }
}

impl IntoEvent for CTMediaKeyCode {
    type Target = MediaKeyCode;

    fn into_event(self) -> Self::Target {
        match self {
            CTMediaKeyCode::Play => MediaKeyCode::Play,
            CTMediaKeyCode::Pause => MediaKeyCode::Pause,
            CTMediaKeyCode::PlayPause => MediaKeyCode::PlayPause,
            CTMediaKeyCode::Reverse => MediaKeyCode::Reverse,
            CTMediaKeyCode::Stop => MediaKeyCode::Stop,
            CTMediaKeyCode::FastForward => MediaKeyCode::FastForward,
            CTMediaKeyCode::Rewind => MediaKeyCode::Rewind,
            CTMediaKeyCode::TrackNext => MediaKeyCode::TrackNext,
            CTMediaKeyCode::TrackPrevious => MediaKeyCode::TrackPrevious,
            CTMediaKeyCode::Record => MediaKeyCode::Record,
            CTMediaKeyCode::LowerVolume => MediaKeyCode::LowerVolume,
            CTMediaKeyCode::RaiseVolume => MediaKeyCode::RaiseVolume,
            CTMediaKeyCode::MuteVolume => MediaKeyCode::MuteVolume,
        }
    }
}

impl IntoEvent for CTModifierKeyCode {
    type Target = ModifierKeyCode;

    fn into_event(self) -> Self::Target {
        match self {
            CTModifierKeyCode::LeftShift => ModifierKeyCode::LeftShift,
            CTModifierKeyCode::LeftControl => ModifierKeyCode::LeftControl,
            CTModifierKeyCode::LeftAlt => ModifierKeyCode::LeftAlt,
            CTModifierKeyCode::LeftSuper => ModifierKeyCode::LeftSuper,
            CTModifierKeyCode::LeftHyper => ModifierKeyCode::LeftHyper,
            CTModifierKeyCode::LeftMeta => ModifierKeyCode::LeftMeta,
            CTModifierKeyCode::RightShift => ModifierKeyCode::RightShift,
            CTModifierKeyCode::RightControl => ModifierKeyCode::RightControl,
            CTModifierKeyCode::RightAlt => ModifierKeyCode::RightAlt,
            CTModifierKeyCode::RightSuper => ModifierKeyCode::RightSuper,
            CTModifierKeyCode::RightHyper => ModifierKeyCode::RightHyper,
            CTModifierKeyCode::RightMeta => ModifierKeyCode::RightMeta,
            CTModifierKeyCode::IsoLevel3Shift => ModifierKeyCode::IsoLevel3Shift,
            CTModifierKeyCode::IsoLevel5Shift => ModifierKeyCode::IsoLevel5Shift,
        }
    }
}