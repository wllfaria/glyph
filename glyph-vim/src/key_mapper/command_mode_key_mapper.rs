use glyph_core::event_loop::event::{KeyCode, KeyEvent};
use glyph_core::key_mapper::Command;

use crate::key_mapper::{CommandWrapper, VimCommand};

#[derive(Debug)]
pub struct CommandModeKeymapper {
    command: String,
}

impl CommandModeKeymapper {
    pub fn new() -> Self {
        Self {
            command: String::new(),
        }
    }

    pub fn dock_height(&self) -> u16 {
        if self.command.is_empty() { 0 } else { 1 }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Vec<CommandWrapper> {
        match key.code {
            KeyCode::Char(c) => {
                self.command.push(c);
                vec![]
            }
            KeyCode::Backspace => {
                if self.command.is_empty() {
                    return vec![CommandWrapper::Vim(VimCommand::NormalMode)];
                }

                self.command.pop();
                vec![]
            }
            KeyCode::Enter => self.handle_command(),
            KeyCode::Left => vec![],
            KeyCode::Right => vec![],
            KeyCode::Up => vec![],
            KeyCode::Down => vec![],
            KeyCode::Home => vec![],
            KeyCode::End => vec![],
            KeyCode::PageUp => vec![],
            KeyCode::PageDown => vec![],
            KeyCode::Tab => vec![],
            KeyCode::BackTab => vec![],
            KeyCode::Delete => vec![],
            KeyCode::Insert => vec![],
            KeyCode::F(_) => vec![],
            KeyCode::Null => vec![],
            KeyCode::Esc => vec![],
            KeyCode::CapsLock => vec![],
            KeyCode::ScrollLock => vec![],
            KeyCode::NumLock => vec![],
            KeyCode::PrintScreen => vec![],
            KeyCode::Pause => vec![],
            KeyCode::Menu => vec![],
            KeyCode::KeypadBegin => vec![],
            KeyCode::Media(_) => vec![],
            KeyCode::Modifier(_) => vec![],
        }
    }

    fn handle_command(&mut self) -> Vec<CommandWrapper> {
        if self.command.is_empty() {
            return vec![];
        }

        match self.command.as_str() {
            "q" => vec![CommandWrapper::General(Command::Quit)],
            "quit" => vec![CommandWrapper::General(Command::Quit)],
            "w" => vec![CommandWrapper::General(Command::Save)],
            "write" => vec![CommandWrapper::General(Command::Save)],
            "wq" => vec![
                CommandWrapper::General(Command::Save),
                CommandWrapper::General(Command::Quit),
            ],
            "waq" => vec![
                CommandWrapper::General(Command::SaveAll),
                CommandWrapper::General(Command::Quit),
            ],
            _ => vec![],
        }
    }
}
