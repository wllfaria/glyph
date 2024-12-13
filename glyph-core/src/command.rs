use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;

use parking_lot::RwLock;
use tree_sitter::{InputEdit, Point};

use crate::cursor::Cursor;
use crate::editor::{Editor, Mode};
use crate::syntax::Highlighter;
use crate::window::WindowId;

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

    if cursor.y() - window.scroll().1 > area.height.into() {
        window.scroll_y_to(cursor.y() - area.height as usize + 1);
    }
}

fn move_to_sof(ctx: &mut Context) {
    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window_id = tab.tree.focus();
    let window = tab.tree.window_mut(window_id);

    let mut cursors = ctx.cursors.write();
    let cursor = cursors.get_mut(&window_id).unwrap();
    cursor.move_to(0, 0);

    if cursor.y().saturating_sub(window.scroll().1) == 0 {
        window.scroll_y_to(window.scroll().1 - window.scroll().1 - cursor.y());
    }
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
