use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::cursor::Cursor;
use crate::editor::Editor;
use crate::window::WindowId;

#[derive(Debug)]
pub struct Context<'ctx> {
    pub editor: Arc<RwLock<Editor>>,
    pub cursors: &'ctx mut BTreeMap<WindowId, Cursor>,
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
        let cursor = ctx.cursors.get_mut(&window.id).unwrap();
        cursor.move_left();
    }

    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();
    let window = tab.tree.window_mut(window);
    let cursor = ctx.cursors.get_mut(&window.id).unwrap();

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
        let cursor = ctx.cursors.get_mut(&window.id).unwrap();
        cursor.move_down(document);
    }

    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();
    let window = tab.tree.window_mut(window);
    let cursor = ctx.cursors.get_mut(&window.id).unwrap();

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
        let cursor = ctx.cursors.get_mut(&window.id).unwrap();
        cursor.move_up();
    }

    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();
    let window = tab.tree.window_mut(window);
    let cursor = ctx.cursors.get_mut(&window.id).unwrap();

    if cursor.y().checked_sub(window.scroll().1).is_none() {
        window.scroll_up();
    }
}

pub fn move_right(ctx: &mut Context) {
    {
        let editor = ctx.editor.read();
        let tab = editor.focused_tab();
        let window = tab.tree.focus();
        let window = tab.tree.window(window);
        let document = editor.document(&window.document);
        let cursor = ctx.cursors.get_mut(&window.id).unwrap();
        cursor.move_right(document);
    }

    let mut editor = ctx.editor.write();
    let tab = editor.focused_tab_mut();
    let window = tab.tree.focus();
    let window = tab.tree.window_mut(window);
    let cursor = ctx.cursors.get_mut(&window.id).unwrap();

    if cursor.x() - window.scroll().0 >= window.area.width.into() {
        window.scroll_right();
    }
}
