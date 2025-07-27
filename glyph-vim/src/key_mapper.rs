use glyph_core::event_loop::Event;
use glyph_core::event_loop::event::{KeyCode, KeyEvent};
use glyph_core::key_mapper::{Command, EditorMode, Keymapper, ResolvedKeymap, VimMode};
use glyph_trie::Trie;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum CommandWrapper {
    General(Command),
    Vim(VimCommand),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum VimCommand {
    InsertMode,
    NormalMode,
    CommandMode,
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
    normal_keymaps: Trie<Keymap>,
    insert_keymaps: Trie<Keymap>,
    editor_mode: VimMode,
    command: String,
}

impl VimKeymapper {
    pub fn new() -> Self {
        let loaded_keymaps = load_vim_keymaps();

        Self {
            normal_keymaps: loaded_keymaps.normal,
            insert_keymaps: loaded_keymaps.insert,
            buffered_key: String::new(),
            editor_mode: VimMode::Normal,
            command: String::new(),
        }
    }

    fn create_keymap(&self, commands: Vec<Command>) -> ResolvedKeymap {
        ResolvedKeymap {
            commands,
            mode: Some(self.mode()),
        }
    }

    fn handle_normal_mode_key(&mut self, key: KeyEvent) -> Option<ResolvedKeymap> {
        let key_str = key.to_string();
        let full_key = format!("{}{}", self.buffered_key, key_str);
        let query = self.normal_keymaps.get(&full_key);

        // if the key is just a part of a bigger keymap, buffer the key and do nothing
        if query.continues {
            self.buffered_key.push_str(&key_str);
            return None;
        }

        let Some(keymap) = query.value else {
            self.buffered_key.clear();
            return None;
        };

        assert!(keymap.mode == self.editor_mode);

        let mut commands = vec![];
        for command in keymap.commands.iter() {
            match command {
                CommandWrapper::General(command) => commands.push(*command),
                CommandWrapper::Vim(command) => match command {
                    VimCommand::InsertMode => self.editor_mode = VimMode::Insert,
                    VimCommand::NormalMode => self.editor_mode = VimMode::Normal,
                    VimCommand::CommandMode => self.editor_mode = VimMode::Command,
                },
            }
        }

        self.buffered_key.clear();
        Some(self.create_keymap(commands))
    }

    fn handle_insert_mode_key(&mut self, key: KeyEvent) -> Option<ResolvedKeymap> {
        let key_str = key.to_string();
        let full_key = format!("{}{}", self.buffered_key, key_str);
        let query = self.insert_keymaps.get(&full_key);

        // if the key is just a part of a bigger keymap, buffer the key and do nothing
        if query.continues {
            self.buffered_key.push_str(&key_str);
            return None;
        }

        // if the query doesn't continue, the keymap has no value and there's have a buffered key,
        // it means that the string started with a keymap-path but terminated with a non-valid
        // keymap. So return every buffered character as a character to be inserted
        if query.value.is_none() && !self.buffered_key.is_empty() {
            self.buffered_key.push_str(&key_str);
            let commands = self.buffered_key.chars().map(Command::TypeChar).collect();
            return Some(self.create_keymap(commands));
        }

        match query.value {
            Some(keymap) => {
                assert!(keymap.mode == self.editor_mode);

                let mut commands = vec![];
                for command in keymap.commands.iter() {
                    match command {
                        CommandWrapper::General(command) => commands.push(*command),
                        CommandWrapper::Vim(command) => match command {
                            VimCommand::InsertMode => self.editor_mode = VimMode::Insert,
                            VimCommand::NormalMode => self.editor_mode = VimMode::Normal,
                            VimCommand::CommandMode => self.editor_mode = VimMode::Command,
                        },
                    }
                }

                self.buffered_key.clear();
                Some(self.create_keymap(commands))
            }
            None => match key.code {
                KeyCode::Backspace => Some(self.create_keymap(vec![Command::DeletePrevChar])),
                KeyCode::Left => Some(self.create_keymap(vec![Command::MoveCursorLeft])),
                KeyCode::Down => Some(self.create_keymap(vec![Command::MoveCursorDown])),
                KeyCode::Up => Some(self.create_keymap(vec![Command::MoveCursorUp])),
                KeyCode::Right => Some(self.create_keymap(vec![Command::MoveCursorRight])),
                KeyCode::Home => Some(self.create_keymap(vec![Command::MoveCursorLineStart])),
                KeyCode::End => Some(self.create_keymap(vec![Command::MoveCursorLineEnd])),
                KeyCode::PageUp => Some(self.create_keymap(vec![Command::PageUp])),
                KeyCode::PageDown => Some(self.create_keymap(vec![Command::PageDown])),
                KeyCode::Delete => Some(self.create_keymap(vec![Command::DeleteCurrChar])),
                KeyCode::Char(c) => Some(self.create_keymap(vec![Command::TypeChar(c)])),
                KeyCode::Esc => {
                    self.editor_mode = VimMode::Normal;
                    None
                }
                KeyCode::Enter => None,
                KeyCode::Tab => None,
                KeyCode::BackTab => None,
                KeyCode::Insert => None,
                KeyCode::F(_) => None,
                KeyCode::Null => None,
                KeyCode::CapsLock => None,
                KeyCode::ScrollLock => None,
                KeyCode::NumLock => None,
                KeyCode::PrintScreen => None,
                KeyCode::Pause => None,
                KeyCode::Menu => None,
                KeyCode::KeypadBegin => None,
                KeyCode::Media(_) => None,
                KeyCode::Modifier(_) => None,
            },
        }
    }

    fn handle_command_mode_key(&mut self, key: KeyEvent) -> Option<ResolvedKeymap> {
        match key.code {
            KeyCode::Char(c) => {
                self.command.push(c);
                None
            }
            KeyCode::Backspace => {
                if self.command.is_empty() {
                    self.editor_mode = VimMode::Normal;
                    return None;
                }

                self.command.pop();
                None
            }
            KeyCode::Enter => None,
            KeyCode::Left => None,
            KeyCode::Right => None,
            KeyCode::Up => None,
            KeyCode::Down => None,
            KeyCode::Home => None,
            KeyCode::End => None,
            KeyCode::PageUp => None,
            KeyCode::PageDown => None,
            KeyCode::Tab => None,
            KeyCode::BackTab => None,
            KeyCode::Delete => None,
            KeyCode::Insert => None,
            KeyCode::F(_) => None,
            KeyCode::Null => None,
            KeyCode::Esc => None,
            KeyCode::CapsLock => None,
            KeyCode::ScrollLock => None,
            KeyCode::NumLock => None,
            KeyCode::PrintScreen => None,
            KeyCode::Pause => None,
            KeyCode::Menu => None,
            KeyCode::KeypadBegin => None,
            KeyCode::Media(media_key_code) => None,
            KeyCode::Modifier(modifier_key_code) => None,
        }
    }
}

impl Keymapper for VimKeymapper {
    fn mode(&self) -> EditorMode {
        EditorMode::Vim(self.editor_mode)
    }

    fn parse_event(&mut self, event: Option<Event>) -> Option<ResolvedKeymap> {
        let Event::Key(key) = event?;

        match self.editor_mode {
            VimMode::Normal => self.handle_normal_mode_key(key),
            VimMode::Insert => self.handle_insert_mode_key(key),
            VimMode::Command => self.handle_command_mode_key(key),
            VimMode::Visual => None,
        }
    }
}

struct LoadedKeymaps {
    normal: Trie<Keymap>,
    insert: Trie<Keymap>,
}

// TODO: this will probably not cut it when it comes to repetitions of keymaps. Such as d10j or
// 10dj, will we need a parser for commands?
fn load_vim_keymaps() -> LoadedKeymaps {
    let mut normal_keymaps = Trie::new();
    let mut insert_keymaps = Trie::new();

    let normal = VimMode::Normal;
    let insert = VimMode::Insert;

    let move_cursor_left = CommandWrapper::General(Command::MoveCursorLeft);
    let move_cursor_down = CommandWrapper::General(Command::MoveCursorDown);
    let move_cursor_up = CommandWrapper::General(Command::MoveCursorUp);
    let move_cursor_right = CommandWrapper::General(Command::MoveCursorRight);
    let move_cursor_right_over_lines = CommandWrapper::General(Command::MoveCursorRightOverLines);
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
    let delete_prev_char = CommandWrapper::General(Command::DeletePrevChar);
    let delete_curr_char = CommandWrapper::General(Command::DeleteCurrChar);
    let move_to_next_word = CommandWrapper::General(Command::MoveToNextWord);
    let quit = CommandWrapper::General(Command::Quit);

    let enter_insert_mode = CommandWrapper::Vim(VimCommand::InsertMode);
    let enter_normal_mode = CommandWrapper::Vim(VimCommand::NormalMode);
    let enter_command_mode = CommandWrapper::Vim(VimCommand::CommandMode);

    // cursor movement motions
    normal_keymaps.insert("h", Keymap::new(normal, vec![move_cursor_left]));
    normal_keymaps.insert("j", Keymap::new(normal, vec![move_cursor_down]));
    normal_keymaps.insert("k", Keymap::new(normal, vec![move_cursor_up]));
    normal_keymaps.insert("l", Keymap::new(normal, vec![move_cursor_right]));
    normal_keymaps.insert("0", Keymap::new(normal, vec![move_cursor_to_line_start]));
    normal_keymaps.insert("$", Keymap::new(normal, vec![move_cursor_to_line_end]));
    normal_keymaps.insert("gg", Keymap::new(normal, vec![move_to_top]));
    normal_keymaps.insert("G", Keymap::new(normal, vec![move_to_bottom]));
    normal_keymaps.insert("%", Keymap::new(normal, vec![move_to_matching_pair]));
    normal_keymaps.insert("^", Keymap::new(normal, vec![move_to_first_non_space]));
    normal_keymaps.insert("g_", Keymap::new(normal, vec![move_to_last_non_space]));
    normal_keymaps.insert("}", Keymap::new(normal, vec![move_to_next_paragraph]));
    normal_keymaps.insert("{", Keymap::new(normal, vec![move_to_prev_paragraph]));
    normal_keymaps.insert("w", Keymap::new(normal, vec![move_to_next_word]));

    normal_keymaps.insert(
        "<cr>",
        Keymap::new(normal, vec![move_cursor_down, move_to_first_non_space]),
    );

    normal_keymaps.insert(" ", Keymap::new(normal, vec![move_cursor_right_over_lines]));

    normal_keymaps.insert("X", Keymap::new(normal, vec![delete_prev_char]));
    normal_keymaps.insert("x", Keymap::new(normal, vec![delete_curr_char]));

    normal_keymaps.insert("i", Keymap::new(normal, vec![enter_insert_mode]));
    normal_keymaps.insert("q", Keymap::new(normal, vec![quit]));
    normal_keymaps.insert("dd", Keymap::new(normal, vec![delete_whole_line]));
    normal_keymaps.insert("<c-u>", Keymap::new(normal, vec![page_up]));
    normal_keymaps.insert("<c-d>", Keymap::new(normal, vec![page_down]));
    normal_keymaps.insert(":", Keymap::new(normal, vec![enter_command_mode]));

    // insert mode keymaps
    insert_keymaps.insert("<esc>", Keymap::new(insert, vec![enter_normal_mode]));

    insert_keymaps.insert("<c-right>", Keymap::new(insert, vec![move_to_next_word]));
    insert_keymaps.insert("<left>", Keymap::new(insert, vec![move_cursor_left]));
    insert_keymaps.insert("<down>", Keymap::new(insert, vec![move_cursor_down]));
    insert_keymaps.insert("<up>", Keymap::new(insert, vec![move_cursor_up]));
    insert_keymaps.insert("<right>", Keymap::new(insert, vec![move_cursor_right]));
    insert_keymaps.insert("<c-c>", Keymap::new(insert, vec![enter_normal_mode]));

    LoadedKeymaps {
        normal: normal_keymaps,
        insert: insert_keymaps,
    }
}