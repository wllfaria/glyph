use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;

use parking_lot::RwLock;
use ropey::Rope;
use tree_sitter::{InputEdit, Point};

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
}

pub enum MappableCommand {
    Static {
        name: &'static str,
        fun: fn(ctx: &mut Context),
    },
    Dynamic {
        callback: Box<dyn Fn()>,
    },
}

macro_rules! static_cmd {
    ($($name:ident,)*) => {
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

fn move_left(ctx: &mut Context) {
    {
        let editor = ctx.editor.read();
        let tab = editor.focused_tab();
        let window = tab.tree.focus();
        let window = tab.tree.window(window);
        let mut cursors = ctx.cursors.write();
        let cursor = cursors.get_mut(&window.id).unwrap();
        cursor.move_left();
    }

    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();
    let window = tab.tree.window_mut(window);
    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window.id).unwrap();

    if cursor.x().checked_sub(window.scroll().0).is_none() {
        window.scroll_left();
    }
}

fn move_down(ctx: &mut Context) {
    {
        let editor = ctx.editor.read();
        let tab = editor.focused_tab();
        let window = tab.tree.focus();
        let window = tab.tree.window(window);
        let document = editor.document(&window.document);
        let mut cursors = ctx.cursors.write();
        let cursor = cursors.get_mut(&window.id).unwrap();
        cursor.move_down(document);
    }

    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();
    let window = tab.tree.window_mut(window);
    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window.id).unwrap();

    if cursor.y() - window.scroll().1 >= window.area.height.into() {
        window.scroll_down();
    }
}

fn move_up(ctx: &mut Context) {
    {
        let editor = ctx.editor.read();
        let tab = editor.focused_tab();
        let window = tab.tree.focus();
        let window = tab.tree.window(window);
        let mut cursors = ctx.cursors.write();
        let cursor = cursors.get_mut(&window.id).unwrap();
        cursor.move_up();
    }

    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();
    let window = tab.tree.window_mut(window);
    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window.id).unwrap();

    if cursor.y().checked_sub(window.scroll().1).is_none() {
        window.scroll_up();
    }
}

fn move_right(ctx: &mut Context) {
    {
        let editor = ctx.editor.read();
        let tab = editor.focused_tab();
        let window = tab.tree.focus();
        let window = tab.tree.window(window);
        let document = editor.document(&window.document);
        let mut cursors = ctx.cursors.write();
        let cursor = cursors.get_mut(&window.id).unwrap();
        cursor.move_right(document);
    }

    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();
    let window = tab.tree.window_mut(window);
    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window.id).unwrap();

    if cursor.x() - window.scroll().0 >= window.area.width.into() {
        window.scroll_right();
    }
}

fn remove_curr_char(ctx: &mut Context) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(&document);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window).expect("window has no cursor");

    let text = document.text_mut();

    let start_char = text.line_to_char(cursor.y());
    let col = start_char + cursor.x();

    let start_byte = text.char_to_byte(col);
    let end_byte = text.char_to_byte(col + 1);

    text.remove(col..col + 1);

    let syntax = ctx
        .highlighter
        .trees
        .get_mut(&document.id)
        .expect("document syntax is not registered on highlighter");

    if let Some(tree) = &mut syntax.tree {
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

        tree.edit(&edit);
        ctx.highlighter.update_document(document);
    }
}

fn remove_prev_char(ctx: &mut Context) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(&document);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window).expect("window has no cursor");

    let text = document.text_mut();

    let start_char = text.line_to_char(cursor.y());
    let col = start_char + cursor.x();

    let start_byte = text.char_to_byte(col.saturating_sub(1));
    let end_byte = text.char_to_byte(col);

    text.remove(col.saturating_sub(1)..col);

    let syntax = ctx
        .highlighter
        .trees
        .get_mut(&document.id)
        .expect("document syntax is not registered on highlighter");

    if let Some(tree) = &mut syntax.tree {
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

        tree.edit(&edit);
        ctx.highlighter.update_document(document);
    }
}

fn delete_line(ctx: &mut Context) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(&document);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window).expect("window has no cursor");

    let text = document.text_mut();

    let line_start_char = text.line_to_char(cursor.y());
    let line_end_char = text.line_to_char(cursor.y() + 1);
    let line_start_byte = text.line_to_byte(cursor.y());
    let line_end_byte = text.line_to_byte(cursor.y() + 1);
    let total_lines = text.len_lines();

    text.remove(line_start_char..line_end_char);

    let syntax = ctx
        .highlighter
        .trees
        .get_mut(&document.id)
        .expect("document syntax is not registered on highlighter");

    if let Some(tree) = &mut syntax.tree {
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

        tree.edit(&edit);
        ctx.highlighter.update_document(document);
    }

    if cursor.y() >= total_lines - 1 {
        cursor.move_to(cursor.x(), total_lines - 2);
    }
}

fn move_to_eof(ctx: &mut Context) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window_id = tab.tree.focus();

    let area = tab.tree.window_mut(window_id).area;
    let document = tab.tree.window_mut(window_id).document;

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window_id).unwrap();

    let document = editor.document_mut(&document);
    let text = document.text_mut();
    let y = text.len_lines() - 1;
    let last_line = text.line(y);
    let x = last_line.len_chars();

    cursor.move_to(x, y);

    let tab = editor.focused_tab_mut();
    let window = tab.tree.window_mut(window_id);

    snap_scroll_down(cursor, window, area);
}

fn move_to_sof(ctx: &mut Context) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window_id = tab.tree.focus();
    let window = tab.tree.window_mut(window_id);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window_id).unwrap();
    cursor.move_to(0, 0);

    snap_scroll_up(cursor, window);
}

fn move_to_sol(ctx: &mut Context) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window_id = tab.tree.focus();
    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window_id).unwrap();
    cursor.move_to(0, cursor.y());
}

fn move_to_eol(ctx: &mut Context) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window_id = tab.tree.focus();

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window_id).unwrap();

    let document = tab.tree.window_mut(window_id).document;
    let document = editor.document(&document);
    let line_len = document.text().line(cursor.y()).len_chars();

    cursor.move_to(line_len - 1, cursor.y());
}

fn page_down(ctx: &mut Context) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window_id = tab.tree.focus();

    let area = tab.tree.window_mut(window_id).area;
    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window_id).unwrap();

    let document = tab.tree.window_mut(window_id).document;
    let max_line = editor.document(&document).text().len_lines();

    let half_page = area.height / 2;
    let next_stop = (cursor.y() + half_page as usize).min(max_line - 1);

    let tab = editor.focused_tab_mut();
    let window = tab.tree.window_mut(window_id);
    cursor.move_to(cursor.x(), next_stop);
    snap_scroll_down(cursor, window, area);
}

fn page_up(ctx: &mut Context) {
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

fn insert_line_above(ctx: &mut Context) {
    let mut editor = ctx.editor.write();
    editor.set_mode(Mode::Insert);

    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(&document);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window).expect("window has no cursor");

    let line_break = document.line_break;
    let text = document.text_mut();

    let line_start_char = text.line_to_char(cursor.y());
    let line_start_byte = text.line_to_byte(cursor.y());

    text.insert(line_start_char, line_break.as_ref());

    let syntax = ctx
        .highlighter
        .trees
        .get_mut(&document.id)
        .expect("document syntax is not registered on highlighter");

    if let Some(tree) = &mut syntax.tree {
        let edit = InputEdit {
            start_byte: line_start_byte,
            old_end_byte: line_start_byte,
            new_end_byte: line_start_byte + line_break.as_ref().len(),
            start_position: Point::new(cursor.y(), 0),
            old_end_position: Point::new(cursor.y(), 0),
            new_end_position: Point::new(cursor.y() + 1, 0),
        };

        tree.edit(&edit);
        ctx.highlighter.update_document(document);
    }
}

fn insert_line_below(ctx: &mut Context) {
    let mut editor = ctx.editor.write();
    editor.set_mode(Mode::Insert);

    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(&document);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window).expect("window has no cursor");

    let line_break = document.line_break;
    let text = document.text_mut();

    let line_start_char = text.line_to_char(cursor.y() + 1);
    let line_start_byte = text.line_to_byte(cursor.y() + 1);

    text.insert(line_start_char, line_break.as_ref());
    cursor.move_down(document);

    let syntax = ctx
        .highlighter
        .trees
        .get_mut(&document.id)
        .expect("document syntax is not registered on highlighter");

    if let Some(tree) = &mut syntax.tree {
        let edit = InputEdit {
            start_byte: line_start_byte,
            old_end_byte: line_start_byte,
            new_end_byte: line_start_byte + line_break.as_ref().len(),
            start_position: Point::new(cursor.y() + 1, 0),
            old_end_position: Point::new(cursor.y() + 1, 0),
            new_end_position: Point::new(cursor.y() + 2, 0),
        };

        tree.edit(&edit);
        ctx.highlighter.update_document(document);
    }
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

    TextQuery {
        start_line: cursor.y(),
        start_col: cursor.x(),
        end_line: 0,
        end_col: 0,
    }
}

fn delete_word(ctx: &mut Context) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();

    let document = tab.tree.window_mut(window).document;
    let document = editor.document_mut(&document);

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
        let start_byte = text.char_to_byte(start_char);

        let old_end_byte = line_start_byte + line_len_bytes;
        let new_end_byte = start_byte;
        let start_position = Point::new(cursor.y(), cursor.x());
        let old_end_position = Point::new(cursor.y(), line_len_chars);
        let new_end_position = start_position;

        let start_char = text.line_to_char(cursor.y()) + query.start_col;
        let end_char = text.line_to_char(cursor.y() + 1);
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

    tracing::debug!("same line");

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

fn edit_tree(ctx: &mut Context, document: DocumentId, edit: InputEdit) {
    let mut editor = ctx.editor.write();
    let document = editor.document_mut(&document);

    tracing::debug!("{edit:#?}");

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

fn insert_at_eol(ctx: &mut Context) {
    move_to_eol(ctx);
    insert_mode(ctx);
}

fn insert_ahead(ctx: &mut Context) {
    move_right(ctx);
    insert_mode(ctx);
}

fn insert_mode(ctx: &mut Context) {
    ctx.editor.write().set_mode(Mode::Insert)
}

fn normal_mode(ctx: &mut Context) {
    ctx.editor.write().set_mode(Mode::Normal)
}

fn command_mode(ctx: &mut Context) {
    ctx.editor.write().set_mode(Mode::Command)
}

fn snap_scroll_down(cursor: &Cursor, window: &mut Window, area: Rect) {
    if cursor.y() - window.scroll().1 >= area.height.into() {
        window.scroll_y_to(cursor.y() - area.height as usize + 1);
    }
}

fn snap_scroll_up(cursor: &Cursor, window: &mut Window) {
    if cursor.y().saturating_sub(window.scroll().1) == 0 {
        window.scroll_y_to(window.scroll().1 - (window.scroll().1 - cursor.y()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;
    use crate::editor::OpenAction;

    fn setup_test<'a>(text: &str, highlighter: &'a mut Highlighter, cursor: Cursor) -> Context<'a> {
        let mut editor = Editor::new((0, 0, 10, 10));
        let (_, doc) = editor.new_file_with_document(".", text.to_string(), OpenAction::SplitVertical);
        let mut cursors = BTreeMap::new();
        cursors.insert(WindowId::new(1).unwrap(), cursor);
        let document = editor.document(&doc);
        highlighter.add_document(document);

        Context {
            editor: Arc::new(RwLock::new(editor)),
            cursors: Arc::new(RwLock::new(cursors)),
            highlighter,
        }
    }

    fn with_document<'a, F>(ctx: &'a Context<'a>, f: F)
    where
        F: Fn(&Document),
    {
        let editor = ctx.editor.read();
        let win = editor.focused_tab().tree.focus();
        let doc = editor.focused_tab().tree.window(win).document;
        let document = editor.document(&doc);
        f(document);
    }

    #[test]
    fn test_delete_word() {
        let mut highlighter = Highlighter::new();
        let mut ctx = setup_test(
            "local function create_inspector(opts)\n",
            &mut highlighter,
            Cursor::new(0, 0),
        );

        delete_word(&mut ctx);

        with_document(&ctx, |document| {
            let text = document.text();
            assert_eq!(text.to_string(), "function create_inspector(opts)\n")
        });

        let mut highlighter = Highlighter::new();
        let mut ctx = setup_test(
            "local function create_inspector(opts)\n",
            &mut highlighter,
            Cursor::new(15, 0),
        );

        delete_word(&mut ctx);

        with_document(&ctx, |document| {
            let text = document.text();
            assert_eq!(text.to_string(), "local function _inspector(opts)\n")
        });

        let mut highlighter = Highlighter::new();
        let mut ctx = setup_test(
            "local function create_inspector(opts)\n",
            &mut highlighter,
            Cursor::new(21, 0),
        );

        delete_word(&mut ctx);

        with_document(&ctx, |document| {
            let text = document.text();
            assert_eq!(text.to_string(), "local function createinspector(opts)\n")
        });
    }
}
