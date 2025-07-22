use glyph_trie::Trie;

use crate::event_loop::event::Event;
use crate::key_mapper::{Command, Keymapper};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
enum CommandWrapper {
    General(Command),
}

#[derive(Debug)]
pub struct Keymap {
    commands: Vec<CommandWrapper>,
}

impl Keymap {
    fn new(commands: Vec<CommandWrapper>) -> Self {
        Self { commands }
    }
}

#[derive(Debug, Default)]
pub struct VSCodeKeymapper {
    keymaps: Trie<Keymap>,
}

impl VSCodeKeymapper {
    pub fn new() -> Self {
        let keymaps = load_vsc_keymaps();
        Self { keymaps }
    }
}

impl Keymapper for VSCodeKeymapper {
    fn parse_event(&mut self, event: Option<Event>) -> Option<Vec<Command>> {
        let Event::Key(key) = event?;
        let key_str = key.to_string();

        let query = self.keymaps.get(&key_str)?;
        let keymap = query.value?;

        let mut commands = vec![];
        for command in keymap.commands.iter() {
            match command {
                CommandWrapper::General(command) => commands.push(command.clone()),
            }
        }

        Some(commands)
    }
}

fn load_vsc_keymaps() -> Trie<Keymap> {
    let mut keymaps = Trie::new();

    let move_cursor_left = CommandWrapper::General(Command::MoveCursorLeft);
    let move_cursor_down = CommandWrapper::General(Command::MoveCursorDown);
    let move_cursor_up = CommandWrapper::General(Command::MoveCursorUp);
    let move_cursor_right = CommandWrapper::General(Command::MoveCursorRight);
    let quit = CommandWrapper::General(Command::Quit);

    keymaps.insert("q", Keymap::new(vec![quit]));
    keymaps.insert("<left>", Keymap::new(vec![move_cursor_left]));
    keymaps.insert("<down>", Keymap::new(vec![move_cursor_down]));
    keymaps.insert("<up>", Keymap::new(vec![move_cursor_up]));
    keymaps.insert("<right>", Keymap::new(vec![move_cursor_right]));

    keymaps
}
