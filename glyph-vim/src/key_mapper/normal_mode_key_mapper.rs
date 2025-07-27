use glyph_core::event_loop::event::KeyEvent;
use glyph_trie::Trie;

use super::Keymap;
use crate::key_mapper::CommandWrapper;

#[derive(Debug, Default)]
pub struct NormalModeKeymapper {
    buffered_key: String,
    normal_keymaps: Trie<Keymap>,
}

impl NormalModeKeymapper {
    pub fn new(normal_keymaps: Trie<Keymap>) -> Self {
        Self {
            normal_keymaps,
            buffered_key: String::new(),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Vec<CommandWrapper> {
        let key_str = key.to_string();
        let full_key = format!("{}{}", self.buffered_key, key_str);
        let query = self.normal_keymaps.get(&full_key);

        // if the key is just a part of a bigger keymap, buffer the key and do nothing
        if query.continues {
            self.buffered_key.push_str(&key_str);
            return vec![];
        }

        let Some(keymap) = query.value else {
            self.buffered_key.clear();
            return vec![];
        };

        self.buffered_key.clear();
        keymap.commands.clone()
    }
}