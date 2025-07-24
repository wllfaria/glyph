use glyph_core::event_loop::Event;
use glyph_core::key_mapper::{Command, EditorMode, Keymapper, ResolvedKeymap, VimMode};
use glyph_trie::Trie;

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
    mode: VimMode,
    commands: Vec<CommandWrapper>,
}

impl Keymap {
    fn new(mode: VimMode, commands: Vec<CommandWrapper>) -> Self {
        Self { mode, commands }
    }
}

#[derive(Debug, Default)]
pub struct VimKeymapper {
    buffered_key: String,
    keymaps: Trie<Keymap>,
    editor_mode: VimMode,
}

impl VimKeymapper {
    pub fn new() -> Self {
        let keymaps = load_vim_keymaps();

        Self {
            keymaps,
            buffered_key: String::new(),
            editor_mode: VimMode::Normal,
        }
    }
}

impl Keymapper for VimKeymapper {
    fn parse_event(&mut self, event: Option<Event>) -> Option<ResolvedKeymap> {
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
            return None;
        }

        let Some(keymap) = query.value else {
            self.buffered_key.clear();
            return None;
        };

        if keymap.mode != self.editor_mode {
            self.buffered_key.clear();
            return None;
        }

        let mut commands = vec![];
        for command in keymap.commands.iter() {
            match command {
                CommandWrapper::General(command) => commands.push(command.clone()),
                CommandWrapper::Vim(command) => match command {
                    VimCommand::InsertMode => self.editor_mode = VimMode::Insert,
                    VimCommand::NormalMode => self.editor_mode = VimMode::Normal,
                },
            }
        }

        self.buffered_key.clear();

        Some(ResolvedKeymap {
            commands,
            mode: Some(EditorMode::Vim(self.editor_mode)),
        })
    }
}

fn load_vim_keymaps() -> Trie<Keymap> {
    let mut keymaps = Trie::new();

    let normal = VimMode::Normal;
    let insert = VimMode::Insert;

    let move_cursor_left = CommandWrapper::General(Command::MoveCursorLeft);
    let move_cursor_down = CommandWrapper::General(Command::MoveCursorDown);
    let move_cursor_up = CommandWrapper::General(Command::MoveCursorUp);
    let move_cursor_right = CommandWrapper::General(Command::MoveCursorRight);
    let move_cursor_to_line_start = CommandWrapper::General(Command::MoveCursorLineStart);
    let move_cursor_to_line_end = CommandWrapper::General(Command::MoveCursorLineEnd);
    let delete_whole_line = CommandWrapper::General(Command::DeleteWholeLine);
    let move_to_top = CommandWrapper::General(Command::MoveToTop);
    let move_to_bottom = CommandWrapper::General(Command::MoveToBottom);
    let page_up = CommandWrapper::General(Command::PageUp);
    let page_down = CommandWrapper::General(Command::PageDown);
    let move_to_matching_pair = CommandWrapper::General(Command::MoveToMatchingPair);
    let move_to_first_non_space = CommandWrapper::General(Command::MoveToFirstNonSpace);
    let move_to_last_non_space = CommandWrapper::General(Command::MoveToLastNonSpace);
    let move_to_next_paragraph = CommandWrapper::General(Command::MoveToNextParagraph);
    let move_to_prev_paragraph = CommandWrapper::General(Command::MoveToPrevParagraph);
    let quit = CommandWrapper::General(Command::Quit);

    let enter_insert_mode = CommandWrapper::Vim(VimCommand::InsertMode);
    let enter_normal_mode = CommandWrapper::Vim(VimCommand::NormalMode);

    // cursor movement motions
    keymaps.insert("h", Keymap::new(normal, vec![move_cursor_left]));
    keymaps.insert("j", Keymap::new(normal, vec![move_cursor_down]));
    keymaps.insert("k", Keymap::new(normal, vec![move_cursor_up]));
    keymaps.insert("l", Keymap::new(normal, vec![move_cursor_right]));
    keymaps.insert("0", Keymap::new(normal, vec![move_cursor_to_line_start]));
    keymaps.insert("$", Keymap::new(normal, vec![move_cursor_to_line_end]));
    keymaps.insert("gg", Keymap::new(normal, vec![move_to_top]));
    keymaps.insert("G", Keymap::new(normal, vec![move_to_bottom]));
    keymaps.insert("%", Keymap::new(normal, vec![move_to_matching_pair]));
    keymaps.insert("^", Keymap::new(normal, vec![move_to_first_non_space]));
    keymaps.insert("g_", Keymap::new(normal, vec![move_to_last_non_space]));
    keymaps.insert("}", Keymap::new(normal, vec![move_to_next_paragraph]));
    keymaps.insert("{", Keymap::new(normal, vec![move_to_prev_paragraph]));

    keymaps.insert("i", Keymap::new(normal, vec![enter_insert_mode]));
    keymaps.insert("q", Keymap::new(normal, vec![quit]));
    keymaps.insert("dd", Keymap::new(normal, vec![delete_whole_line]));
    keymaps.insert("<c-u>", Keymap::new(normal, vec![page_up]));
    keymaps.insert("<c-d>", Keymap::new(normal, vec![page_down]));

    keymaps.insert("<esc>", Keymap::new(insert, vec![enter_normal_mode]));

    keymaps
}