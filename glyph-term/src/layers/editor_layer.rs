use glyph_config::{GlyphConfig, GutterAnchor};
use glyph_core::document::Document;
use glyph_core::editor::Editor;
use glyph_core::highlights::HighlightGroup;
use glyph_core::rect::{Point, Rect};
use glyph_core::window::Window;

use crate::backend::{Cell, CursorKind};
use crate::buffer::Buffer;
use crate::renderer::{DrawContext, RenderLayer};
use crate::ui::line_number::{get_line_drawer, LineNumberDrawer};

#[derive(Debug, Default)]
pub struct EditorLayer {}

impl EditorLayer {
    pub fn new() -> EditorLayer {
        EditorLayer {}
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_window(
        &self,
        mut area: Rect,
        ctx: &mut DrawContext,
        document: &Document,
        window: &Window,
        buffer: &mut Buffer,
        config: GlyphConfig,
    ) {
        if config.gutter().enabled {
            let gutter_size = calculate_gutter_size(document, config);
            let gutter_area = match config.gutter().anchor {
                GutterAnchor::Left => area.split_left(gutter_size),
                GutterAnchor::Right => area.split_right(gutter_size),
            };
            self.draw_gutter(gutter_area, document, window, buffer, config);
        }
        self.draw_document(area, ctx, document, window, buffer, config);
    }

    pub fn draw_document(
        &self,
        area: Rect,
        ctx: &mut DrawContext,
        document: &Document,
        window: &Window,
        buffer: &mut Buffer,
        config: GlyphConfig,
    ) {
        let text = document.text();

        let start_byte = text.line_to_byte(window.cursor().y() + window.scroll().1);
        let end_byte = text.line_to_byte(window.cursor().y() + window.scroll().1 + area.height as usize + 1);
        let text = document.text().slice(start_byte..end_byte);
        let start = text.line_to_char(window.cursor().y());

        for (y, line) in text.lines_at(start).take(area.height as usize).enumerate() {
            let mut style = HighlightGroup::default();

            for (x, ch) in line.chars().enumerate() {
                if let Some(syntax) = ctx.highlighter.document_syntax(document.id) {
                    if let Some(captures) = syntax.captures.get(&y) {
                        if let Some(capture) = captures.iter().find(|c| x >= c.start.column && x < c.end.column) {
                            if let Some(group) = config.highlight_groups.get(&capture.name) {
                                style = *group
                            }
                        }
                    }
                }

                if x >= area.width.into() {
                    break;
                };

                buffer.set_cell(area.x + x as u16, y as u16, Cell::new(ch), style)
            }

            for x in line.chars().count()..area.width as usize {
                buffer.set_cell(area.x + x as u16, y as u16, Cell::new(' '), style);
            }
        }
    }

    pub fn draw_gutter(
        &self,
        area: Rect,
        document: &Document,
        window: &Window,
        buffer: &mut Buffer,
        config: GlyphConfig,
    ) {
        let line_drawer = get_line_drawer(&config);
        line_drawer.draw_line_numbers(area, document, window, buffer, config);
    }

    pub fn draw_statusline(&self, buffer: &mut Buffer, ctx: &mut DrawContext, area: Rect) {
        let tab = ctx.editor.get_focused_tab();
        let focused_window = tab.tree.focus();
        let window = tab.tree.get(focused_window);

        let editor_mode = ctx.editor.mode().to_string().to_uppercase();
        let cursor = window.cursor().to_string();

        let padding = area.width - (editor_mode.len() + cursor.len()) as u16;
        let gap = " ".repeat(padding as usize - 6);
        let statusline = format!(" [ {editor_mode} ]{gap}{cursor} ");

        buffer.set_string(area.x, area.y, &statusline);
    }
}

impl RenderLayer for EditorLayer {
    fn draw(&self, buffer: &mut Buffer, ctx: &mut DrawContext, config: GlyphConfig) {
        let mut area = ctx.editor.area();
        let mut statusline_area = area.split_bottom(2);
        let _commandline_area = statusline_area.split_bottom(1);

        self.draw_statusline(buffer, ctx, statusline_area);

        for (window, _) in ctx.editor.get_focused_tab().tree.windows() {
            let document = ctx.editor.document(&window.document).unwrap();
            self.draw_window(area, ctx, document, window, buffer, config);
        }
    }

    fn cursor(&self, editor: &Editor, config: GlyphConfig) -> (Option<Point>, CursorKind) {
        let tab = editor.get_focused_tab();
        let focused_window = tab.tree.focus();
        let window = tab.tree.get(focused_window);
        let document = editor.document(&window.document).unwrap();
        let gutter_size = calculate_gutter_size(document, config);

        let point = Point {
            x: (window.cursor().x() as u16 + gutter_size),
            y: window.cursor().y() as u16,
        };
        (Some(point), CursorKind::Block)
    }
}

fn calculate_gutter_size(document: &Document, config: GlyphConfig) -> u16 {
    let total_lines = document.text().len_lines();
    // +1 is to always accomodate bigger lines
    let lines_length = digits_in_number(total_lines) as u16 + 1;
    let lines_length = u16::max(lines_length, 3) + config.gutter().sign_column.size();
    // +1 is the padding before document
    lines_length + 1
}

#[inline]
pub fn digits_in_number(number: usize) -> usize {
    (f32::log10(number as f32) + 1.0) as usize
}
