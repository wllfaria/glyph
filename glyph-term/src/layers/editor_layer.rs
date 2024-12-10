use std::collections::BTreeMap;

use crossterm::event::{Event, KeyCode, KeyEvent};
use glyph_config::{GlyphConfig, GutterAnchor};
use glyph_core::command::Context as CmdContext;
use glyph_core::cursor::Cursor;
use glyph_core::document::Document;
use glyph_core::editor::{EventResult, Mode};
use glyph_core::highlights::HighlightGroup;
use glyph_core::rect::{Point, Rect};
use glyph_core::syntax::Highlighter;
use glyph_core::window::{Window, WindowId};

use crate::backend::CursorKind;
use crate::buffer::Buffer;
use crate::renderer::{Context, RenderLayer};
use crate::ui::line_number::{get_line_drawer, LineNumberDrawer};

#[derive(Debug, Default)]
pub struct EditorLayer;

impl EditorLayer {
    pub fn new() -> EditorLayer {
        EditorLayer {}
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_window(
        &self,
        mut area: Rect,
        document: &Document,
        window: &Window,
        buffer: &mut Buffer,
        highlighter: &Highlighter,
        cursors: &BTreeMap<WindowId, Cursor>,
        config: GlyphConfig,
    ) {
        if config.gutter().enabled {
            let gutter_size = calculate_gutter_size(document, config);
            let gutter_area = match config.gutter().anchor {
                GutterAnchor::Left => area.split_left(gutter_size),
                GutterAnchor::Right => area.split_right(gutter_size),
            };
            self.draw_gutter(gutter_area, document, window, cursors, buffer, config);
        }
        self.draw_document(area, document, window, buffer, highlighter, config);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_document(
        &self,
        area: Rect,
        document: &Document,
        window: &Window,
        buffer: &mut Buffer,
        highlighter: &Highlighter,
        config: GlyphConfig,
    ) {
        let text = document.text();
        let start_byte = text.line_to_byte(window.scroll().1);
        let end_byte = text.len_lines();
        let end_byte = end_byte.min(window.scroll().1 + area.height as usize);
        let total_lines = end_byte - window.scroll().1;
        let end_byte = text.line_to_byte(end_byte);
        let text = document.text().slice(start_byte..end_byte);
        let start = text.line_to_char(0);

        for (y, line) in text.lines_at(start).take(total_lines).enumerate() {
            let mut style = HighlightGroup::default();

            for (x, ch) in line.chars().enumerate() {
                if let Some(syntax) = highlighter.document_syntax(document.id) {
                    if let Some(captures) = syntax.captures.get(&(y + window.scroll().1)) {
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

                match ch {
                    '\n' | '\r' => buffer.set_cell(area.x + x as u16, y as u16, ' ', style),
                    _ => buffer.set_cell(area.x + x as u16, y as u16, ch, style),
                }
            }

            for x in line.chars().count()..area.width as usize {
                buffer.set_cell(area.x + x as u16, y as u16, ' ', style);
            }
        }
    }

    pub fn draw_gutter(
        &self,
        area: Rect,
        document: &Document,
        window: &Window,
        cursors: &BTreeMap<WindowId, Cursor>,
        buffer: &mut Buffer,
        config: GlyphConfig,
    ) {
        let cursor = cursors.get(&window.id).unwrap();
        let line_drawer = get_line_drawer(config);
        line_drawer.draw_line_numbers(area, document, window, cursor, buffer, config);
    }

    pub fn draw_statusline(&self, buffer: &mut Buffer, ctx: &mut Context, area: Rect) {
        let editor = ctx.editor.read();
        let tab = editor.focused_tab();
        let focused_window = tab.tree.focus();
        let window = tab.tree.window(focused_window);

        let cursor = ctx.cursors.get(&window.id).unwrap();
        let editor_mode = ctx.editor.read().mode().to_string().to_uppercase();
        let cursor = cursor.to_string();

        let padding = area.width - (editor_mode.len() + cursor.len()) as u16;
        let gap = " ".repeat(padding as usize - 6);
        let statusline = format!(" [ {editor_mode} ]{gap}{cursor} ");

        buffer.set_string(area.x, area.y, &statusline, HighlightGroup::default());
    }

    pub fn handle_key_event(
        &self,
        key_event: &KeyEvent,
        ctx: &mut Context,
        config: GlyphConfig,
    ) -> Result<Option<EventResult>, std::io::Error> {
        match key_event.code {
            KeyCode::Char(_) => match ctx.editor.read().mode() {
                Mode::Normal => {}
                Mode::Insert => {}
            },
            _ => todo!(),
        }
        if let KeyCode::Char(ch) = key_event.code {
            if let Some(result) = config.keymaps.find_word(ch.to_string()) {
                if result.data.mode == ctx.editor.read().mode() {
                    let mut context = CmdContext {
                        editor: ctx.editor.clone(),
                        cursors: ctx.cursors,
                    };
                    result.data.command.run(&mut context);
                }
            }
        }

        Ok(None)
    }
}

impl RenderLayer for EditorLayer {
    fn draw(&self, buffer: &mut Buffer, ctx: &mut Context, config: GlyphConfig) {
        let mut area = ctx.editor.read().area();
        let mut statusline_area = area.split_bottom(2);
        let _commandline_area = statusline_area.split_bottom(1);

        self.draw_statusline(buffer, ctx, statusline_area);

        let editor = ctx.editor.read();
        for (window, _) in editor.focused_tab().tree.windows() {
            let document = editor.document(&window.document);
            self.draw_window(area, document, window, buffer, ctx.highlighter, ctx.cursors, config);
        }
    }

    fn cursor(&self, ctx: &mut Context, config: GlyphConfig) -> (Option<Point>, CursorKind) {
        let editor = ctx.editor.read();
        let tab = editor.focused_tab();
        let focused_window = tab.tree.focus();
        let window = tab.tree.window(focused_window);
        let cursor = ctx.cursors.get(&window.id).unwrap();
        let document = editor.document(&window.document);
        let gutter_size = calculate_gutter_size(document, config);

        let point = Point {
            x: ((cursor.x() + gutter_size as usize) - window.scroll().0) as u16,
            y: (cursor.y() - window.scroll().1) as u16,
        };

        (Some(point), CursorKind::Block)
    }

    fn handle_event(
        &self,
        event: &Event,
        ctx: &mut Context,
        config: GlyphConfig,
    ) -> Result<Option<EventResult>, std::io::Error> {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event, ctx, config),
            _ => Ok(None),
        }
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
