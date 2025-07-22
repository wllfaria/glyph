use glyph_trie::Trie;

use crate::event_loop::event::{Event, KeyCode};
use crate::key_mapper::{Command, Keymapper};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum EditorMode {
    #[default]
    Normal,
    Insert,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
enum CommandWrapper {
    General(Command),
    Vim(VimCommand),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum VimCommand {
    InsertMode,
    NormalMode,
}

#[derive(Debug)]
pub struct Keymap {
    mode: EditorMode,
    commands: Vec<CommandWrapper>,
}

impl Keymap {
    fn new(mode: EditorMode, commands: Vec<CommandWrapper>) -> Self {
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
        let key_str = key.to_string();

        let full_key = format!("{}{}", self.buffered_key, key_str);
        let Some(query) = self.keymaps.get(&full_key) else {
            self.buffered_key.clear();
            return None;
        };

        // TODO: if the query has a value but continues, we need to store the action
        // and continue buffering until either a timeout is reached (triggering the action)
        // or the user presses another key.
        if query.continues {
            self.buffered_key.push_str(&key_str);
        }

        let Some(keymap) = query.value else {
            self.buffered_key.clear();
            return None;
        };

        if keymap.mode != self.editor_mode {
            return None;
        }

        let mut commands = vec![];
        for command in keymap.commands.iter() {
            match command {
                CommandWrapper::General(command) => commands.push(command.clone()),
                CommandWrapper::Vim(command) => match command {
                    VimCommand::InsertMode => self.editor_mode = EditorMode::Insert,
                    VimCommand::NormalMode => self.editor_mode = EditorMode::Normal,
                },
            }
        }

        Some(commands)
    }
}

fn load_vim_keymaps() -> Trie<Keymap> {
    let mut keymaps = Trie::new();

    let normal_mode = EditorMode::Normal;
    let insert_mode = EditorMode::Insert;

    let move_cursor_left = CommandWrapper::General(Command::MoveCursorLeft);
    let move_cursor_down = CommandWrapper::General(Command::MoveCursorDown);
    let move_cursor_up = CommandWrapper::General(Command::MoveCursorUp);
    let move_cursor_right = CommandWrapper::General(Command::MoveCursorRight);
    let quit = CommandWrapper::General(Command::Quit);

    let enter_insert_mode = CommandWrapper::Vim(VimCommand::InsertMode);
    let enter_normal_mode = CommandWrapper::Vim(VimCommand::NormalMode);

    keymaps.insert("i", Keymap::new(normal_mode, vec![enter_insert_mode]));
    keymaps.insert("q", Keymap::new(normal_mode, vec![quit]));
    keymaps.insert("h", Keymap::new(normal_mode, vec![move_cursor_left]));
    keymaps.insert("j", Keymap::new(normal_mode, vec![move_cursor_down]));
    keymaps.insert("k", Keymap::new(normal_mode, vec![move_cursor_up]));
    keymaps.insert("l", Keymap::new(normal_mode, vec![move_cursor_right]));

    keymaps.insert("<esc>", Keymap::new(insert_mode, vec![enter_normal_mode]));

    keymaps
}
