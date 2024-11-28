use glyph_term::backend::{Cell, CursorKind};
use glyph_term::buffer::Buffer;
use glyph_term::graphics::Rect;

use crate::document::Document;
use crate::editor::Editor;
use crate::renderer::{Anchor, DrawContext, RenderLayer};
use crate::window::Window;

#[derive(Debug, Default)]
pub struct EditorLayer {}

impl EditorLayer {
    pub fn new() -> EditorLayer {
        EditorLayer {}
    }

    pub fn draw_window(
        &self,
        editor: &Editor,
        area: Rect,
        document: &Document,
        window: &Window,
        buffer: &mut Buffer,
        is_focused: bool,
    ) {
        self.draw_document(area, document, window, buffer);
    }

    pub fn draw_document(&self, area: Rect, document: &Document, window: &Window, buffer: &mut Buffer) {
        let text = document.text().slice(..);
        let start = text.line_to_char(window.cursor.y());

        for (y, line) in text.lines_at(start).take(area.height as usize).enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if x >= area.width.into() {
                    break;
                };

                buffer.set_cell(x as u16, y as u16, Cell::new(ch))
            }

            for x in line.chars().count()..area.width as usize {
                buffer.set_cell(x as u16, y as u16, Cell::new(' '));
            }
        }
    }

    pub fn draw_statusline(&self, buffer: &mut Buffer, ctx: &mut DrawContext, area: Rect) {
        let tab = ctx.editor.get_focused_tab();
        let focused_window = tab.tree.focus();
        let window = tab.tree.get(focused_window);

        let editor_mode = ctx.editor.mode().to_string().to_uppercase();
        let cursor = window.cursor.to_string();

        let padding = area.width - (editor_mode.len() + cursor.len()) as u16;
        let gap = " ".repeat(padding as usize - 6);
        let statusline = format!(" [ {editor_mode} ]{gap}{cursor} ");

        buffer.set_string(area.x, area.y, statusline);
    }
}

impl RenderLayer for EditorLayer {
    fn draw(&self, buffer: &mut Buffer, ctx: &mut DrawContext) {
        let mut area = ctx.editor.area();
        let mut statusline_area = area.split_bottom(2);
        let _commandline_area = statusline_area.split_bottom(1);

        self.draw_statusline(buffer, ctx, statusline_area);

        for (window, is_focused) in ctx.editor.get_focused_tab().tree.windows() {
            let document = ctx.editor.document(&window.document).unwrap();
            self.draw_window(ctx.editor, area, document, window, buffer, is_focused);
        }
    }

    fn cursor(&self, editor: &Editor) -> (Option<Anchor>, CursorKind) {
        let tab = editor.get_focused_tab();
        let focused_window = tab.tree.focus();
        let window = tab.tree.get(focused_window);
        (Some(window.cursor.into()), CursorKind::Block)
    }
}
