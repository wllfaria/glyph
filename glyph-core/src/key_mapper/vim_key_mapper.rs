use glyph_trie::Trie;

use crate::event_loop::event::{Event, KeyCode};
use crate::key_mapper::{Command, Keymapper};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum EditorMode {
    #[default]
    Normal,
    Insert,
}

#[derive(Debug)]
pub struct Keymap {
    mode: EditorMode,
    commands: Vec<Command>,
}

impl Keymap {
    pub fn new(mode: EditorMode, commands: Vec<Command>) -> Self {
        Self { mode, commands }
    }
}

#[derive(Debug, Default)]
pub struct VimKeymapper {
    buffered_key: String,
    keymaps: Trie<Keymap>,
    editor_mode: EditorMode,
}

impl VimKeymapper {
    pub fn new() -> Self {
        let keymaps = load_vim_keymaps();

        Self {
            keymaps,
            buffered_key: String::new(),
            editor_mode: EditorMode::Normal,
        }
    }
}

impl Keymapper for VimKeymapper {
    fn parse_event(&mut self, event: Option<Event>) -> Option<Vec<Command>> {
        let Event::Key(key) = event?;
        let KeyCode::Char(c) = key.code else { return None };

        let full_key = format!("{}{}", self.buffered_key, c);
        let Some(query) = self.keymaps.get(&full_key) else {
            self.buffered_key.clear();
            return None;
        };

        // TODO: if the query has a value but continues, we need to store the action
        // and continue buffering until either a timeout is reached (triggering the action)
        // or the user presses another key.
        if query.continues {
            self.buffered_key.push(c);
        }

        let Some(keymap) = query.value else {
            self.buffered_key.clear();
            return None;
        };

        if keymap.mode != self.editor_mode {
            return None;
        }

        Some(keymap.commands.clone())
    }
}

fn load_vim_keymaps() -> Trie<Keymap> {
    let mut keymaps = Trie::new();

    let normal_mode = EditorMode::Normal;

    let move_cursor_left = Command::MoveCursorLeft;
    let move_cursor_down = Command::MoveCursorDown;
    let move_cursor_up = Command::MoveCursorUp;
    let move_cursor_right = Command::MoveCursorRight;

    keymaps.insert("h", Keymap::new(normal_mode, vec![move_cursor_left]));
    keymaps.insert("j", Keymap::new(normal_mode, vec![move_cursor_down]));
    keymaps.insert("k", Keymap::new(normal_mode, vec![move_cursor_up]));
    keymaps.insert("l", Keymap::new(normal_mode, vec![move_cursor_right]));

    keymaps
}
