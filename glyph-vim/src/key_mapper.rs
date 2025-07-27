mod insert_mode_key_mapper;
mod normal_mode_key_mapper;

use glyph_core::key_mapper::{Command, VimMode};
use glyph_trie::Trie;
pub use insert_mode_key_mapper::InsertModeKeymapper;
pub use normal_mode_key_mapper::NormalModeKeymapper;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum CommandWrapper {
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

pub struct LoadedKeymaps {
    pub normal: Trie<Keymap>,
    pub insert: Trie<Keymap>,
}

// TODO: this will probably not cut it when it comes to repetitions of keymaps. Such as d10j or
// 10dj, will we need a parser for commands?
pub fn load_vim_keymaps() -> LoadedKeymaps {
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