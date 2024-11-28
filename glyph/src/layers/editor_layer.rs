use glyph_term::backend::Cell;
use glyph_term::buffer::Buffer;

use crate::document::Document;
use crate::editor::Editor;
use crate::renderer::{DrawContext, RenderLayer};
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
        document: &Document,
        window: &Window,
        buffer: &mut Buffer,
        is_focused: bool,
    ) {
        self.draw_document(document, window, buffer);
    }

    pub fn draw_document(&self, document: &Document, window: &Window, buffer: &mut Buffer) {
        let text = document.text().slice(..);
        let start = text.line_to_char(window.cursor.y());

        for (y, line) in text.lines_at(start).take(window.area.height as usize - 1).enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if x >= window.area.width.into() {
                    break;
                };

                buffer.set_cell(x, y, Cell::new(ch))
            }

            for x in line.chars().count()..window.area.width as usize {
                buffer.set_cell(x, y, Cell::new(' '));
            }
        }
    }
}

impl RenderLayer for EditorLayer {
    fn draw(&self, buffer: &mut Buffer, ctx: &mut DrawContext) {
        for (window, is_focused) in ctx.editor.get_focused_tab().tree.windows() {
            let document = ctx.editor.document(&window.document).unwrap();
            self.draw_window(ctx.editor, document, window, buffer, is_focused);
        }
    }
}
