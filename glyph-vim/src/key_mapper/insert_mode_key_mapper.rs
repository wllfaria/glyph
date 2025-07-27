use glyph_core::event_loop::event::{KeyCode, KeyEvent};
use glyph_core::key_mapper::Command;
use glyph_trie::Trie;

use crate::key_mapper::{CommandWrapper, Keymap, VimCommand};

#[derive(Debug)]
pub struct InsertModeKeymapper {
    buffered_key: String,
    insert_keymaps: Trie<Keymap>,
}

impl InsertModeKeymapper {
    pub fn new(insert_keymaps: Trie<Keymap>) -> Self {
        Self {
            buffered_key: String::new(),
            insert_keymaps,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Vec<CommandWrapper> {
        let key_str = key.to_string();
        let full_key = format!("{}{}", self.buffered_key, key_str);
        let query = self.insert_keymaps.get(&full_key);

        // if the key is just a part of a bigger keymap, buffer the key and do nothing
        if query.continues {
            self.buffered_key.push_str(&key_str);
            return vec![];
        }

        // if the query doesn't continue, the keymap has no value and there's have a buffered key,
        // it means that the string started with a keymap-path but terminated with a non-valid
        // keymap. So return every buffered character as a character to be inserted
        if query.value.is_none() && !self.buffered_key.is_empty() {
            self.buffered_key.push_str(&key_str);
            return self
                .buffered_key
                .chars()
                .map(|ch| CommandWrapper::General(Command::TypeChar(ch)))
                .collect();
        }

        match query.value {
            Some(keymap) => keymap.commands.clone(),
            None => match key.code {
                KeyCode::Backspace => vec![CommandWrapper::General(Command::DeletePrevChar)],
                KeyCode::Left => vec![CommandWrapper::General(Command::MoveCursorLeft)],
                KeyCode::Down => vec![CommandWrapper::General(Command::MoveCursorDown)],
                KeyCode::Up => vec![CommandWrapper::General(Command::MoveCursorUp)],
                KeyCode::Right => vec![CommandWrapper::General(Command::MoveCursorRight)],
                KeyCode::Home => vec![CommandWrapper::General(Command::MoveCursorLineStart)],
                KeyCode::End => vec![CommandWrapper::General(Command::MoveCursorLineEnd)],
                KeyCode::PageUp => vec![CommandWrapper::General(Command::PageUp)],
                KeyCode::PageDown => vec![CommandWrapper::General(Command::PageDown)],
                KeyCode::Delete => vec![CommandWrapper::General(Command::DeleteCurrChar)],
                KeyCode::Char(c) => vec![CommandWrapper::General(Command::TypeChar(c))],
                KeyCode::Esc => vec![CommandWrapper::Vim(VimCommand::InsertMode)],
                KeyCode::Enter => vec![],
                KeyCode::Tab => vec![],
                KeyCode::BackTab => vec![],
                KeyCode::Insert => vec![],
                KeyCode::F(_) => vec![],
                KeyCode::Null => vec![],
                KeyCode::CapsLock => vec![],
                KeyCode::ScrollLock => vec![],
                KeyCode::NumLock => vec![],
                KeyCode::PrintScreen => vec![],
                KeyCode::Pause => vec![],
                KeyCode::Menu => vec![],
                KeyCode::KeypadBegin => vec![],
                KeyCode::Media(_) => vec![],
                KeyCode::Modifier(_) => vec![],
            },
        }
    }
}