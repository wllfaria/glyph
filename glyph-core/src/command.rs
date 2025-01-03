use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;

use parking_lot::RwLock;
use ropey::Rope;
use tree_sitter::{InputEdit, Point};

use crate::config::GlyphConfig;
use crate::cursor::Cursor;
use crate::document::DocumentId;
use crate::editor::{Editor, Mode};
use crate::rect::Rect;
use crate::syntax::Highlighter;
use crate::window::{Window, WindowId};

trait GlyphChar {
    fn is_linebreak(&self) -> bool;
}

impl GlyphChar for char {
    fn is_linebreak(&self) -> bool {
        matches!(self, '\n' | '\r')
    }
}

#[derive(Debug)]
pub struct Context<'ctx> {
    pub editor: Arc<RwLock<Editor>>,
    pub cursors: Arc<RwLock<BTreeMap<WindowId, Cursor>>>,
    pub highlighter: &'ctx mut Highlighter,
    pub config: GlyphConfig<'ctx>,
}

pub enum MappableCommand {
    Static {
        name: &'static str,
        fun: fn(ctx: &mut Context<'_>),
    },
    Dynamic {
        callback: Box<dyn Fn()>,
    },
}

macro_rules! static_cmd {
    ($($name:ident),* $(,)?) => {
        $(
            #[allow(non_upper_case_globals)]
            pub const $name: Self = Self::Static {
                name: stringify!($name),
                fun: $name,
            };
        )*

        pub const STATIC_CMD_LIST: &'static [Self] = &[
            $(Self::$name,)*
        ];
    };
}

impl MappableCommand {
    static_cmd! {
        move_left,
        move_down,
        move_up,
        move_right,
        delete_line,
        insert_mode,
        normal_mode,
        command_mode,
        move_to_eof,
        move_to_sof,
        move_to_sol,
        move_to_eol,
        page_down,
        page_up,
        insert_line_below,
        insert_line_above,
        insert_at_eol,
        insert_ahead,
        remove_curr_char,
        remove_prev_char,
        delete_word,
        delete_word_prev,
        next_word,
        next_word_big,
        prev_word,
        prev_word_big,
        join_line_below,
    }
}

impl Debug for MappableCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MappableCommand::Static { name, .. } => f
                .debug_struct("Static")
                .field("name", name)
                .field("fun", &"<function>")
                .finish(),
            MappableCommand::Dynamic { .. } => f.debug_struct("Dynamic").field("callback", &"<function>").finish(),
        }
    }
}

enum Direction {
    Left,
    Down,
    Up,
    Right,
}

fn move_cursor(ctx: &mut Context<'_>, dir: Direction) {
    {
        let editor = ctx.editor.read();
        let tab = editor.focused_tab();
        let window = tab.tree.focus();
        let window = tab.tree.window(window);
        let document = editor.document(window.document);
        let mut cursors = ctx.cursors.write();
        let cursor = cursors.get_mut(&window.id).unwrap();

        match dir {
            Direction::Left => cursor.move_left(),
            Direction::Down => cursor.move_down(document),
            Direction::Up => cursor.move_up(document),
            Direction::Right => cursor.move_right(document),
        }
    }

    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();
    let window = tab.tree.window_mut(window);
    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window.id).unwrap();

    let scroll_offset = ctx.config.scroll_offset;

    match dir {
        Direction::Left => {
            if cursor.x().checked_sub(window.scroll().x).is_none() {
                window.scroll_left();
            }
        }
        Direction::Down => {
            if cursor.y() - window.scroll().y + scroll_offset >= window.area.height.into() {
                window.scroll_down();
            }
        }
        Direction::Up => {
            if cursor.y().checked_sub(window.scroll().y + scroll_offset).is_none() {
                window.scroll_up();
            }
        }
        Direction::Right => {
            if cursor.x() - window.scroll().x >= window.area.width.into() {
                window.scroll_right();
            }
        }
    }
}

fn move_left(ctx: &mut Context<'_>) {
    move_cursor(ctx, Direction::Left);
}

fn move_down(ctx: &mut Context<'_>) {
    move_cursor(ctx, Direction::Down);
}

fn move_up(ctx: &mut Context<'_>) {
    move_cursor(ctx, Direction::Up);
}

fn move_right(ctx: &mut Context<'_>) {
    move_cursor(ctx, Direction::Right);
}

fn remove_curr_char(ctx: &mut Context<'_>) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(document);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window).expect("window has no cursor");

    let text = document.text_mut();

    let start_char = text.line_to_char(cursor.y());
    let col = start_char + cursor.x();

    let start_byte = text.char_to_byte(col);
    let end_byte = text.char_to_byte(col + 1);

    text.remove(col..col + 1);

    let document = document.id;
    let edit = InputEdit {
        start_byte,
        old_end_byte: end_byte,
        // after we delete the line, end position is the same as the previous starting
        new_end_byte: start_byte,
        start_position: Point::new(cursor.y(), cursor.x()),
        // end edit at the beginning of the next line
        old_end_position: Point::new(cursor.y(), cursor.x() + 1),
        new_end_position: Point::new(cursor.y(), cursor.x()),
    };

    drop(editor);
    drop(cursors);
    edit_tree(ctx, document, edit);
}

fn join_line_below(ctx: &mut Context<'_>) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window(window).document;
    let document = editor.document_mut(document);

    let cursors = ctx.cursors.read();
    let cursor = cursors.get(&window).unwrap();

    let text = document.text_mut();

    if cursor.y() >= text.len_lines() - 1 {
        return;
    }

    let curr_line_idx = cursor.y();
    let next_line_idx = cursor.y() + 1;

    let line_start = text.line_to_char(curr_line_idx);
    let next_line_start = text.line_to_char(next_line_idx);
    let next_line_end = if next_line_idx + 1 < text.len_lines() {
        text.line_to_char(next_line_idx + 1) - 1 // -1 to not include the next line's newline
    } else {
        text.len_chars()
    };

    let start_byte = text.char_to_byte(line_start + text.line(curr_line_idx).len_chars() - 1);
    let end_byte = text.char_to_byte(next_line_end);

    let next_line_slice = text.slice(next_line_start..next_line_end);
    let next_line_str = next_line_slice.to_string();
    let trimmed_content = next_line_str.trim_start();

    let new_content = format!(" {}", trimmed_content);
    let new_content_bytes = new_content.len();
    let next_line_chars = next_line_slice.len_chars();

    text.remove(line_start + text.line(curr_line_idx).len_chars() - 1..next_line_end);
    text.insert(line_start + text.line(curr_line_idx).len_chars() - 1, &new_content);

    let edit = InputEdit {
        start_byte,
        old_end_byte: end_byte,
        new_end_byte: start_byte + new_content_bytes,
        start_position: Point::new(curr_line_idx, text.line(curr_line_idx).len_chars() - 1),
        old_end_position: Point::new(next_line_idx, next_line_chars),
        new_end_position: Point::new(
            curr_line_idx,
            text.line(curr_line_idx).len_chars() - 1 + new_content.chars().count(),
        ),
    };

    let document = document.id;
    drop(editor);
    drop(cursors);
    edit_tree(ctx, document, edit);
}

pub fn remove_prev_char(ctx: &mut Context<'_>) {
    remove_prev_char_inner(ctx, true);
}

pub fn remove_prev_char_breaking(ctx: &mut Context<'_>) {
    remove_prev_char_inner(ctx, false);
}

fn remove_prev_char_inner(ctx: &mut Context<'_>, stop_at_linebreak: bool) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(document);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window).expect("window has no cursor");

    if cursor.x() == 0 && cursor.y() == 0 {
        return;
    }

    if stop_at_linebreak && cursor.x() == 0 {
        return;
    }

    let text = document.text_mut();

    let start_char = text.line_to_char(cursor.y());
    let col = start_char + cursor.x();

    let start_byte = text.char_to_byte(col.saturating_sub(1));
    let end_byte = text.char_to_byte(col);

    if !stop_at_linebreak && cursor.x() == 0 {
        let len = text.line(cursor.y().saturating_sub(1)).len_chars() - 1;
        cursor.move_to(len, cursor.y().saturating_sub(1));
    } else {
        cursor.move_left();
    }

    text.remove(col.saturating_sub(1)..col);

    let document = document.id;
    let edit = InputEdit {
        start_byte,
        old_end_byte: end_byte,
        // after we delete the line, end position is the same as the previous starting
        new_end_byte: start_byte,
        start_position: Point::new(cursor.y(), cursor.x().saturating_sub(1)),
        // end edit at the beginning of the next line
        old_end_position: Point::new(cursor.y(), cursor.x()),
        new_end_position: Point::new(cursor.y(), cursor.x().saturating_sub(1)),
    };
    drop(editor);
    drop(cursors);

    edit_tree(ctx, document, edit);
}

pub fn break_line(ctx: &mut Context<'_>) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(document);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window).expect("window has no cursor");

    let line_break = document.line_break;
    let text = document.text_mut();

    let start_char = text.line_to_char(cursor.y()) + cursor.x();

    let start_byte = text.char_to_byte(start_char);
    let new_end_byte = start_byte + line_break.as_ref().len();

    text.insert(start_char, line_break.as_ref());
    cursor.move_to(0, cursor.y() + 1);

    let edit = InputEdit {
        start_byte,
        old_end_byte: start_byte,
        new_end_byte,
        start_position: Point::new(cursor.y(), cursor.x()),
        old_end_position: Point::new(cursor.y(), cursor.x()),
        new_end_position: Point::new(cursor.y(), 0),
    };

    let document = document.id;

    drop(editor);
    drop(cursors);
    edit_tree(ctx, document, edit);
}

fn delete_line(ctx: &mut Context<'_>) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(document);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window).expect("window has no cursor");

    let text = document.text_mut();

    let line_start_char = text.line_to_char(cursor.y());
    let line_end_char = text.line_to_char(cursor.y() + 1);
    let line_start_byte = text.line_to_byte(cursor.y());
    let line_end_byte = text.line_to_byte(cursor.y() + 1);
    let total_lines = text.len_lines();

    text.remove(line_start_char..line_end_char);

    if cursor.y() >= total_lines - 1 {
        cursor.move_to(cursor.x(), total_lines - 2);
    }

    let edit = InputEdit {
        start_byte: line_start_byte,
        old_end_byte: line_end_byte,
        // after we delete the line, end position is the same as the previous starting
        new_end_byte: line_start_byte,
        start_position: Point::new(cursor.y(), 0),
        // end edit at the beginning of the next line
        old_end_position: Point::new(cursor.y() + 1, 0),
        new_end_position: Point::new(cursor.y(), 0),
    };

    let document = document.id;

    drop(editor);
    drop(cursors);
    edit_tree(ctx, document, edit);
}

fn move_to_eof(ctx: &mut Context<'_>) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window_id = tab.tree.focus();

    let area = tab.tree.window_mut(window_id).area;
    let document = tab.tree.window_mut(window_id).document;

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window_id).unwrap();

    let document = editor.document_mut(document);
    let text = document.text_mut();
    let y = text.len_lines() - 1;
    let last_line = text.line(y);
    let x = last_line.len_chars();

    cursor.move_to(x, y);

    let tab = editor.focused_tab_mut();
    let window = tab.tree.window_mut(window_id);

    snap_scroll_down(cursor, window, area);
}

fn move_to_sof(ctx: &mut Context<'_>) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window_id = tab.tree.focus();
    let window = tab.tree.window_mut(window_id);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window_id).unwrap();
    cursor.move_to(0, 0);

    snap_scroll_up(cursor, window);
}

fn move_to_sol(ctx: &mut Context<'_>) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window_id = tab.tree.focus();
    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window_id).unwrap();
    cursor.move_to(0, cursor.y());
}

fn move_to_eol(ctx: &mut Context<'_>) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window_id = tab.tree.focus();

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window_id).unwrap();

    let document = tab.tree.window_mut(window_id).document;
    let document = editor.document(document);
    let line_len = document.text().line(cursor.y()).len_chars();

    cursor.move_to(line_len - 2, cursor.y());
}

fn page_down(ctx: &mut Context<'_>) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window_id = tab.tree.focus();

    let area = tab.tree.window_mut(window_id).area;
    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window_id).unwrap();

    let document = tab.tree.window_mut(window_id).document;
    let max_line = editor.document(document).text().len_lines();

    let half_page = area.height / 2;
    let next_stop = (cursor.y() + half_page as usize).min(max_line - 1);

    let tab = editor.focused_tab_mut();
    let window = tab.tree.window_mut(window_id);
    cursor.move_to(cursor.x(), next_stop);
    snap_scroll_down(cursor, window, area);
}

fn page_up(ctx: &mut Context<'_>) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window_id = tab.tree.focus();

    let area = tab.tree.window_mut(window_id).area;
    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window_id).unwrap();

    let half_page = area.height / 2;
    let next_stop = cursor.y().saturating_sub(half_page as usize);

    let window = tab.tree.window_mut(window_id);
    cursor.move_to(cursor.x(), next_stop);

    snap_scroll_up(cursor, window);
}

fn insert_line_above(ctx: &mut Context<'_>) {
    let mut editor = ctx.editor.write();
    editor.set_mode(Mode::Insert);

    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(document);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window).expect("window has no cursor");

    let line_break = document.line_break;
    let text = document.text_mut();

    let line_start_char = text.line_to_char(cursor.y());
    let line_start_byte = text.line_to_byte(cursor.y());

    text.insert(line_start_char, line_break.as_ref());

    let document = document.id;
    let edit = InputEdit {
        start_byte: line_start_byte,
        old_end_byte: line_start_byte,
        new_end_byte: line_start_byte + line_break.as_ref().len(),
        start_position: Point::new(cursor.y(), 0),
        old_end_position: Point::new(cursor.y(), 0),
        new_end_position: Point::new(cursor.y() + 1, 0),
    };

    drop(editor);
    drop(cursors);
    edit_tree(ctx, document, edit);
}

fn next_word(ctx: &mut Context<'_>) {
    next_word_inner(ctx, WordSkip::Small);
}

fn next_word_big(ctx: &mut Context<'_>) {
    next_word_inner(ctx, WordSkip::Big);
}

fn next_word_inner(ctx: &mut Context<'_>, skip: WordSkip) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(document);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window).expect("window has no cursor");

    let text = document.text_mut();

    let query = find_next_word(text, cursor, skip);

    cursor.move_to(query.end_col, query.end_line);
}

fn prev_word(ctx: &mut Context<'_>) {
    prev_word_inner(ctx, WordSkip::Small);
}

fn prev_word_big(ctx: &mut Context<'_>) {
    prev_word_inner(ctx, WordSkip::Big);
}

fn prev_word_inner(ctx: &mut Context<'_>, skip: WordSkip) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(document);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window).expect("window has no cursor");

    let text = document.text_mut();

    let query = find_prev_word(text, cursor, skip);

    cursor.move_to(query.end_col, query.end_line)
}

fn insert_line_below(ctx: &mut Context<'_>) {
    let mut editor = ctx.editor.write();
    editor.set_mode(Mode::Insert);

    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(document);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window).expect("window has no cursor");

    let line_break = document.line_break;
    let text = document.text_mut();

    let line_start_char = text.line_to_char(cursor.y() + 1);
    let line_start_byte = text.line_to_byte(cursor.y() + 1);

    text.insert(line_start_char, line_break.as_ref());
    cursor.move_down(document);
    let document = document.id;
    let edit = InputEdit {
        start_byte: line_start_byte,
        old_end_byte: line_start_byte,
        new_end_byte: line_start_byte + line_break.as_ref().len(),
        start_position: Point::new(cursor.y() + 1, 0),
        old_end_position: Point::new(cursor.y() + 1, 0),
        new_end_position: Point::new(cursor.y() + 2, 0),
    };

    drop(editor);
    drop(cursors);
    edit_tree(ctx, document, edit);
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WordSkip {
    Small,
    Big,
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextQuery {
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
}

impl TextQuery {
    pub fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> TextQuery {
        TextQuery {
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    pub fn is_same_line(&self) -> bool {
        self.start_line == self.end_line
    }
}

/// searches for the next word starting from the current cursor position.
///
/// When searching for the next word, there are a few scenarios that could happen:
/// - Cursor is currently on a space, the next word is defined by the next non-whitespace
///   character, second newline (lf or crlf) in a row, or at the end of the file;
///
/// - Cursor is currently on a word (non-punctuation) character, the next word is defined by the
///   next punctuation character, the next word character after at least one space, or at the end
///   of file
///
/// - Cursor is currently on a punctuacion (special) character, the next word is defined by the
///   next punctuaction character after at least one space, at the next non-punctuation character,
///   or at the end of file
///
/// When skip is `WordSkip::Big` everything still sthe same, except that we always skip until we
/// find a whistespace
fn find_next_word(text: &Rope, cursor: &Cursor, skip: WordSkip) -> TextQuery {
    let mut start_char = text.line_to_char(cursor.y()) + cursor.x();
    let char = text.char(start_char);

    let make_result = |start_char: usize| {
        let end_line = text.char_to_line(start_char);
        let line_start = text.line_to_char(end_line);
        let end_col = start_char - line_start;

        TextQuery {
            start_line: cursor.y(),
            start_col: cursor.x(),
            end_line,
            end_col,
        }
    };

    // when starting from a whitespace, it doesn't matter which kind of jump we are supposed to do,
    // as both small and big jumps would stop at the next word
    if char.is_whitespace() {
        let mut found_newline = false;
        let mut maybe_char = text.get_char(start_char);

        while let Some(char) = maybe_char {
            if char.is_linebreak() && found_newline {
                return make_result(start_char);
            }

            if !char.is_whitespace() {
                return make_result(start_char);
            }

            start_char += 1;
            maybe_char = text.get_char(start_char);
            found_newline = char.is_linebreak();
        }
    }

    match skip {
        WordSkip::Small => {
            let mut maybe_char = text.get_char(start_char);
            let mut found_whitespace = false;
            let in_word = char.is_alphanumeric();
            let in_punctuation = char.is_ascii_punctuation();

            while let Some(char) = maybe_char {
                // regardless of which character we started, if we consumed whitespaces, we should
                // stop on any alphanumeric character
                if char.is_alphanumeric() && found_whitespace {
                    return make_result(start_char);
                }

                match (in_word, in_punctuation) {
                    (true, false) => {
                        if char.is_ascii_punctuation() {
                            return make_result(start_char);
                        }
                    }
                    (false, true) => {
                        if char.is_ascii_punctuation() && found_whitespace {
                            return make_result(start_char);
                        }

                        if char.is_alphanumeric() {
                            return make_result(start_char);
                        }
                    }
                    _ => {}
                }

                if char.is_linebreak() {
                    return make_result(start_char);
                }

                start_char += 1;
                maybe_char = text.get_char(start_char);
                found_whitespace = char.is_whitespace();
            }
        }
        WordSkip::Big => {
            let mut maybe_char = text.get_char(start_char);
            let mut found_whitespace = false;

            while let Some(char) = maybe_char {
                if (char.is_alphanumeric() || char.is_ascii_punctuation()) && found_whitespace {
                    return make_result(start_char);
                }

                start_char += 1;
                maybe_char = text.get_char(start_char);
                found_whitespace = char.is_whitespace();
            }
        }
    }

    unreachable!();
}

/// if we start on a whitespace, the next word is defined by the next non-whitespace
fn find_prev_word(text: &Rope, cursor: &Cursor, skip: WordSkip) -> TextQuery {
    let mut start_char = text.line_to_char(cursor.y()) + cursor.x();
    let char = text.char(start_char);

    let make_result = |start_char: usize| {
        let end_line = text.char_to_line(start_char);
        let line_start = text.line_to_char(end_line);
        let end_col = start_char - line_start;

        TextQuery {
            start_line: cursor.y(),
            start_col: cursor.x(),
            end_line,
            end_col,
        }
    };

    if char.is_whitespace() {
        let mut found_newline = false;
        let mut found_word = false;
        let mut maybe_char = text.get_char(start_char);

        while let Some(char) = maybe_char {
            if char.is_linebreak() && found_newline {
                // after finding two consecutive newlines, we should only count the first one, as
                // the second one marks the end of the word
                return make_result(start_char + 1);
            }

            if char.is_whitespace() && found_word {
                return make_result(start_char);
            }

            if !char.is_whitespace() {
                found_word = true;
            }

            start_char = match start_char.checked_sub(1) {
                Some(n) => n,
                None => return make_result(start_char),
            };
            maybe_char = text.get_char(start_char);
            found_newline = char.is_linebreak();
        }
    }

    if start_char == 0 {
        return make_result(start_char);
    }

    match skip {
        WordSkip::Small => {
            let mut maybe_char = text.get_char(start_char);
            let mut found_whitespace = false;
            let mut found_word = false;
            let in_word = char.is_alphanumeric();
            let in_punctuation = char.is_ascii_punctuation();

            let stop_at_same_word = match (in_word, in_punctuation) {
                // if we are on a word, and the previous char is also a word character, then we
                // stop at the same word
                (true, false) => text
                    .get_char(start_char - 1)
                    .map(|ch| ch.is_alphanumeric())
                    .unwrap_or_default(),
                // if we are on a punctuation, and the previous char is also a punctuation, then we
                // stop at the same word
                (false, true) => text
                    .get_char(start_char - 1)
                    .map(|ch| ch.is_ascii_punctuation())
                    .unwrap_or_default(),
                _ => unreachable!(),
            };

            while let Some(char) = maybe_char {
                match (in_word, in_punctuation) {
                    (true, false) => {
                        if char.is_ascii_punctuation() && stop_at_same_word {
                            return make_result(start_char + 1);
                        }

                        if char.is_ascii_punctuation() {
                            return make_result(start_char);
                        }

                        if char.is_whitespace() {
                            found_whitespace = true;
                        }

                        if char.is_whitespace() && found_word {
                            return make_result(start_char + 1);
                        }

                        if char.is_alphanumeric() && found_whitespace {
                            found_word = true;
                        }
                    }
                    (false, true) => {
                        if !char.is_ascii_punctuation() && stop_at_same_word {
                            return make_result(start_char + 1);
                        }

                        if char.is_alphanumeric() {
                            found_word = true;
                        }

                        if char.is_whitespace() && found_word {
                            return make_result(start_char + 1);
                        }
                    }
                    _ => unreachable!(),
                }

                start_char = match start_char.checked_sub(1) {
                    Some(n) => n,
                    None => return make_result(start_char),
                };
                maybe_char = text.get_char(start_char);
            }
        }
        WordSkip::Big => {
            let mut maybe_char = text.get_char(start_char);
            let mut found_word = false;
            let mut found_whitespace = false;
            let mut found_newline = false;

            let stop_at_same_word = text
                .get_char(start_char - 1)
                .map(|ch| !ch.is_whitespace())
                .unwrap_or_default();

            while let Some(char) = maybe_char {
                if char.is_linebreak() && found_newline {
                    return make_result(start_char + 1);
                }

                if char.is_linebreak() {
                    found_newline = true;
                }

                if char.is_whitespace() && stop_at_same_word {
                    return make_result(start_char + 1);
                }

                if char.is_whitespace() && found_word {
                    return make_result(start_char + 1);
                }

                if char.is_whitespace() {
                    found_whitespace = true;
                }

                if !char.is_whitespace() && found_whitespace {
                    found_word = true;
                }

                if char.is_whitespace() && found_word {
                    return make_result(start_char + 1);
                }

                start_char = match start_char.checked_sub(1) {
                    Some(n) => n,
                    None => return make_result(start_char),
                };
                maybe_char = text.get_char(start_char);
            }
        }
    }

    unreachable!();
}

fn delete_word(ctx: &mut Context<'_>) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(document);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window).expect("window has no cursor");

    let text = document.text_mut();
    let query = find_next_word(text, cursor, WordSkip::Small);

    if !query.is_same_line() {
        let line = text.line(cursor.y());

        let line_start_byte = text.line_to_byte(cursor.y());
        let line_len_bytes = line.len_bytes();
        let line_len_chars = line.len_chars();

        let start_char = text.line_to_char(query.start_line) + query.start_col;
        let end_char = text.line_to_char(cursor.y() + 1);

        let start_byte = text.char_to_byte(start_char);
        let old_end_byte = line_start_byte + line_len_bytes;
        let new_end_byte = start_byte;
        let start_position = Point::new(cursor.y(), cursor.x());
        let old_end_position = Point::new(cursor.y(), line_len_chars);
        let new_end_position = start_position;

        text.remove(start_char..end_char);

        let document = document.id;
        drop(editor);
        drop(cursors);

        return edit_tree(
            ctx,
            document,
            InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte,
                start_position,
                old_end_position,
                new_end_position,
            },
        );
    }

    let start_char = text.line_to_char(query.start_line) + query.start_col;
    let end_char = text.line_to_char(query.end_line) + query.end_col;

    let start_byte = text.char_to_byte(start_char);
    let old_end_byte = text.char_to_byte(end_char);
    let new_end_byte = start_byte;

    let start_position = Point::new(query.start_line, query.start_col);
    let old_end_position = Point::new(query.end_line, query.end_col);
    let new_end_position = start_position;

    text.remove(start_char..end_char);

    let document = document.id;
    drop(editor);
    drop(cursors);

    edit_tree(
        ctx,
        document,
        InputEdit {
            start_byte,
            old_end_byte,
            new_end_byte,
            start_position,
            old_end_position,
            new_end_position,
        },
    );
}

fn delete_word_prev(ctx: &mut Context<'_>) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(document);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window).expect("window has no cursor");

    let text = document.text_mut();
    let query = find_prev_word(text, cursor, WordSkip::Small);

    if !query.is_same_line() {
        let line_start_byte = text.line_to_byte(cursor.y());

        let start_char = text.line_to_char(query.end_line) + query.end_col;
        let end_char = text.line_to_char(query.start_line) + query.start_col;

        let start_byte = text.char_to_byte(start_char);
        let old_end_byte = line_start_byte + cursor.x();
        let new_end_byte = start_byte;
        let start_position = Point::new(query.end_line, query.end_col);
        let old_end_position = Point::new(query.start_line, query.start_col);
        let new_end_position = start_position;

        text.remove(start_char..end_char);
        cursor.move_to(query.end_col, query.end_line);

        let document = document.id;
        drop(editor);
        drop(cursors);

        return edit_tree(
            ctx,
            document,
            InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte,
                start_position,
                old_end_position,
                new_end_position,
            },
        );
    }

    let start_char = text.line_to_char(query.end_line) + query.end_col;
    let end_char = text.line_to_char(query.start_line) + query.start_col;

    let start_byte = text.char_to_byte(start_char);
    let old_end_byte = text.char_to_byte(end_char);
    let new_end_byte = start_byte;

    let start_position = Point::new(query.end_line, query.end_col);
    let old_end_position = Point::new(query.start_line, query.start_col);
    let new_end_position = start_position;

    text.remove(start_char..end_char);
    cursor.move_to(query.end_col, query.end_line);

    let document = document.id;
    drop(editor);
    drop(cursors);

    edit_tree(
        ctx,
        document,
        InputEdit {
            start_byte,
            old_end_byte,
            new_end_byte,
            start_position,
            old_end_position,
            new_end_position,
        },
    );
}

fn edit_tree(ctx: &mut Context<'_>, document: DocumentId, edit: InputEdit) {
    let mut editor = ctx.editor.write();
    let document = editor.document_mut(document);

    let syntax = ctx
        .highlighter
        .trees
        .get_mut(&document.id)
        .expect("document syntax is not registered on highlighter");

    if let Some(tree) = &mut syntax.tree {
        tree.edit(&edit);
        ctx.highlighter.update_document(document);
    }
}

fn insert_at_eol(ctx: &mut Context<'_>) {
    move_to_eol(ctx);
    insert_mode(ctx);
}

fn insert_ahead(ctx: &mut Context<'_>) {
    move_right(ctx);
    insert_mode(ctx);
}

pub fn insert_char(ctx: &mut Context<'_>, ch: char) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab();
    let win = tab.tree.focus();
    let doc = tab.tree.window(win).document;
    let doc = editor.document_mut(doc);

    let text = doc.text_mut();

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&win).unwrap();

    let char_position = text.line_to_char(cursor.y()) + cursor.x();
    text.insert_char(char_position, ch);

    let start_char = text.line_to_char(cursor.y()) + cursor.x();
    let start_byte = text.char_to_byte(start_char);

    let edit = InputEdit {
        start_byte,
        old_end_byte: start_byte,
        new_end_byte: start_byte + ch.len_utf8(),
        start_position: Point::new(cursor.y(), cursor.x()),
        old_end_position: Point::new(cursor.y(), cursor.x()),
        new_end_position: Point::new(cursor.y(), cursor.x() + 1),
    };

    cursor.move_right(doc);

    let document = doc.id;
    drop(editor);
    drop(cursors);
    edit_tree(ctx, document, edit);
}

fn insert_mode(ctx: &mut Context<'_>) {
    ctx.editor.write().set_mode(Mode::Insert)
}

fn normal_mode(ctx: &mut Context<'_>) {
    ctx.editor.write().set_mode(Mode::Normal)
}

fn command_mode(ctx: &mut Context<'_>) {
    ctx.editor.write().set_mode(Mode::Command)
}

fn snap_scroll_down(cursor: &Cursor, window: &mut Window, area: Rect) {
    if cursor.y() - window.scroll().y >= area.height.into() {
        window.scroll_y_to(cursor.y() - area.height as usize + 1);
    }
}

fn snap_scroll_up(cursor: &Cursor, window: &mut Window) {
    if cursor.y().saturating_sub(window.scroll().y) == 0 {
        window.scroll_y_to(window.scroll().y - (window.scroll().y - cursor.y()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::document::Document;
    use crate::editor::OpenAction;

    fn setup_test<'a>(
        text: &str,
        highlighter: &'a mut Highlighter,
        cursor: Cursor,
        config: GlyphConfig<'a>,
    ) -> Context<'a> {
        let mut editor = Editor::new((0, 0, 10, 10));
        let (_, doc) = editor.new_file_with_document(".", text.to_string(), OpenAction::SplitVertical);
        let mut cursors = BTreeMap::new();
        cursors.insert(WindowId::new(1).unwrap(), cursor);
        let document = editor.document(doc);
        highlighter.add_document(document);

        Context {
            editor: Arc::new(RwLock::new(editor)),
            cursors: Arc::new(RwLock::new(cursors)),
            highlighter,
            config,
        }
    }

    fn with_content<'a, F>(ctx: &'a Context<'a>, f: F)
    where
        F: Fn(&Document, &Cursor),
    {
        let editor = ctx.editor.read();
        let win = editor.focused_tab().tree.focus();
        let cursors = ctx.cursors.read();
        let cursor = cursors.get(&win).unwrap();
        let doc = editor.focused_tab().tree.window(win).document;
        let document = editor.document(doc);
        f(document, cursor);
    }

    #[test]
    fn test_delete_word_start() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "local function create_inspector(opts)\n",
            &mut highlighter,
            Cursor::new(0, 0),
            &config,
        );

        delete_word(&mut ctx);

        with_content(&ctx, |document, _| {
            let text = document.text();
            assert_eq!(text.to_string(), "function create_inspector(opts)\n")
        });
    }

    #[test]
    fn test_delete_word_until_underscore() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "local function create_inspector(opts)\n",
            &mut highlighter,
            Cursor::new(15, 0),
            &config,
        );

        delete_word(&mut ctx);

        with_content(&ctx, |document, _| {
            let text = document.text();
            assert_eq!(text.to_string(), "local function _inspector(opts)\n")
        });
    }

    #[test]
    fn test_delete_word_separator() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "local function create_inspector(opts)\n",
            &mut highlighter,
            Cursor::new(21, 0),
            &config,
        );

        delete_word(&mut ctx);

        with_content(&ctx, |document, _| {
            let text = document.text();
            assert_eq!(text.to_string(), "local function createinspector(opts)\n")
        });
    }

    #[test]
    fn test_prev_word_spaces() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "local          function create_inspector(opts)\n",
            &mut highlighter,
            Cursor::new(14, 0),
            &config,
        );

        prev_word(&mut ctx);

        with_content(&ctx, |_, cursor| assert_eq!(cursor, &Cursor::new(0, 0)));
    }
    #[test]
    fn test_prev_word_empty_lines() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "\n\n            local function create_inspector(opts)\n",
            &mut highlighter,
            Cursor::new(11, 2),
            &config,
        );

        prev_word(&mut ctx);

        with_content(&ctx, |_, cursor| assert_eq!(cursor, &Cursor::new(0, 1)));
    }

    #[test]
    fn test_prev_word_partial() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "local function create_inspector(opts)\n",
            &mut highlighter,
            Cursor::new(5, 0),
            &config,
        );

        prev_word(&mut ctx);

        with_content(&ctx, |_, cursor| assert_eq!(cursor, &Cursor::new(0, 0)));
    }

    #[test]
    fn test_prev_word_big_identifier() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "local function create_inspector(opts)\n",
            &mut highlighter,
            Cursor::new(29, 0),
            &config,
        );

        prev_word_big(&mut ctx);

        with_content(&ctx, |_, cursor| assert_eq!(cursor, &Cursor::new(15, 0)));
    }

    #[test]
    fn test_prev_word_big_function() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "local function create_inspector(opts)\n",
            &mut highlighter,
            Cursor::new(15, 0),
            &config,
        );

        prev_word_big(&mut ctx);

        with_content(&ctx, |_, cursor| assert_eq!(cursor, &Cursor::new(6, 0)));
    }

    #[test]
    fn test_prev_word_big_across_lines() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "\nfn is_linebreak(&self) -> bool {\n    matches!(self, '\\n' | '\\r')\n}",
            &mut highlighter,
            Cursor::new(4, 2),
            &config,
        );

        prev_word_big(&mut ctx);

        with_content(&ctx, |_, cursor| assert_eq!(cursor, &Cursor::new(31, 1)));
    }

    #[test]
    fn test_prev_word_big_empty_lines() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "\n\nfn is_linebreak(&self) -> bool {\n    matches!(self, '\\n' | '\\r')\n}",
            &mut highlighter,
            Cursor::new(0, 2),
            &config,
        );

        prev_word_big(&mut ctx);

        with_content(&ctx, |_, cursor| assert_eq!(cursor, &Cursor::new(0, 1)));
    }

    #[test]
    fn test_prev_word_with_ampersand() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "fn is_linebreak(&self) -> bool { }",
            &mut highlighter,
            Cursor::new(14, 0),
            &config,
        );

        prev_word(&mut ctx);

        with_content(&ctx, |_, cursor| assert_eq!(cursor, &Cursor::new(6, 0)));
    }

    #[test]
    fn test_prev_word_short_word() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "fn is_linebreak(&self) -> bool { }",
            &mut highlighter,
            Cursor::new(6, 0),
            &config,
        );

        prev_word(&mut ctx);

        with_content(&ctx, |_, cursor| assert_eq!(cursor, &Cursor::new(5, 0)));
    }

    #[test]
    fn test_prev_word_with_pub() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "pub fn is_linebreak(&self) -> bool { }",
            &mut highlighter,
            Cursor::new(7, 0),
            &config,
        );

        prev_word(&mut ctx);

        with_content(&ctx, |_, cursor| assert_eq!(cursor, &Cursor::new(4, 0)));
    }

    #[test]
    fn test_prev_word_with_colons() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "use crate::config::Config;",
            &mut highlighter,
            Cursor::new(10, 0),
            &config,
        );

        prev_word(&mut ctx);

        with_content(&ctx, |_, cursor| assert_eq!(cursor, &Cursor::new(9, 0)));
    }

    #[test]
    fn test_prev_word_after_keyword() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "pub fn is_linebreak(&self) -> bool { }",
            &mut highlighter,
            Cursor::new(9, 0),
            &config,
        );

        prev_word(&mut ctx);

        with_content(&ctx, |_, cursor| assert_eq!(cursor, &Cursor::new(7, 0)));
    }

    #[test]
    fn test_delete_word_prev() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "pub fn is_linebreak(&self) -> bool { }",
            &mut highlighter,
            Cursor::new(7, 0),
            &config,
        );

        delete_word_prev(&mut ctx);

        with_content(&ctx, |document, _| {
            let text = document.text();
            let expected = "pub is_linebreak(&self) -> bool { }";
            assert_eq!(text.to_string(), expected);
        });
    }

    #[test]
    fn test_delete_word_prev_line() {
        let mut highlighter = Highlighter::new();
        let config = Config::default();
        let mut ctx = setup_test(
            "with_content(&ctx, |document, _| {\n    let text = document.text();\n});",
            &mut highlighter,
            Cursor::new(4, 1),
            &config,
        );

        delete_word_prev(&mut ctx);

        with_content(&ctx, |document, _| {
            let text = document.text();
            let expected = "with_content(&ctx, |document, _| let text = document.text();\n});";
            assert_eq!(text.to_string(), expected);
        });
    }
}
