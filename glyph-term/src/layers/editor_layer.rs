use std::collections::BTreeMap;
use std::sync::Arc;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use glyph_config::{GlyphConfig, GutterAnchor, KeymapConfig};
use glyph_core::command::Context as CmdContext;
use glyph_core::cursor::Cursor;
use glyph_core::document::Document;
use glyph_core::editor::{EventResult, Mode};
use glyph_core::highlights::HighlightGroup;
use glyph_core::rect::{Point, Rect};
use glyph_core::syntax::Highlighter;
use glyph_core::window::{Window, WindowId};
use glyph_runtime::statusline::{StatuslineContent, StatuslineStyle};
use glyph_trie::QueryResult;
use parking_lot::RwLock;

use crate::backend::CursorKind;
use crate::buffer::{Buffer, CellRange, StyleDef};
use crate::renderer::{Context, RenderLayer};
use crate::ui::line_number::{get_line_drawer, LineNumberDrawer};

#[derive(Debug, Default)]
enum KeymapAction {
    #[default]
    None,
    Continue,
    Execute,
}

#[derive(Debug, Default)]
struct KeymapResult<'result> {
    keymap: Option<&'result KeymapConfig<'result>>,
    action: KeymapAction,
}

impl<'result> KeymapResult<'result> {
    pub fn from_query_result(keymap: QueryResult<'result, KeymapConfig<'result>>) -> KeymapResult<'result> {
        match (keymap.continues, keymap.data) {
            (true, _) => KeymapResult {
                keymap: keymap.data,
                action: KeymapAction::Continue,
            },
            (false, None) => KeymapResult {
                keymap: None,
                action: KeymapAction::None,
            },
            (false, Some(keymap)) => KeymapResult {
                keymap: Some(keymap),
                action: KeymapAction::Execute,
            },
        }
    }
}

pub struct StatusSection {
    content: String,
    width: usize,
    style: HighlightGroup,
}

impl StatusSection {
    fn new(content: String, style: HighlightGroup) -> Self {
        Self {
            width: content.chars().count(),
            content,
            style,
        }
    }
}

#[derive(Debug, Default)]
pub struct EditorLayer;

impl EditorLayer {
    pub fn new() -> EditorLayer {
        EditorLayer {}
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_window(
        &self,
        document: &Document,
        window: &Window,
        buffer: &mut Buffer,
        highlighter: &Highlighter,
        cursors: Arc<RwLock<BTreeMap<WindowId, Cursor>>>,
        config: GlyphConfig,
    ) {
        let mut area = window.area;
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

        let start_line = window.scroll().1;
        let visible_lines = area.height as usize;

        let document_lines = text.len_lines();
        let end_line = (start_line + visible_lines).min(document_lines);

        let start_char = text.line_to_char(start_line);
        let end_char = if end_line < document_lines { text.line_to_char(end_line) } else { text.len_chars() };

        let text_slice = text.slice(start_char..end_char);
        let start = text_slice.line_to_char(0);

        // Calculate actual number of lines to render
        let total_lines = end_line - start_line;

        for (y, line) in text_slice.lines_at(start).take(total_lines).enumerate() {
            let mut style = HighlightGroup::default();

            if line.chars().count() == 0 {
                for x in 0..area.width as usize {
                    buffer.set_cell(area.x + x as u16, y as u16, ' ', style);
                }
                continue;
            }

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
                    _ => buffer.set_cell(area.x + x as u16, area.y + y as u16, ch, style),
                }
            }

            for x in line.chars().count()..area.width as usize {
                buffer.set_cell(area.x + x as u16, area.y + y as u16, ' ', style);
            }
        }
    }

    pub fn draw_gutter(
        &self,
        area: Rect,
        document: &Document,
        window: &Window,
        cursors: Arc<RwLock<BTreeMap<WindowId, Cursor>>>,
        buffer: &mut Buffer,
        config: GlyphConfig,
    ) {
        let cursors = cursors.read();
        let cursor = cursors.get(&window.id).expect("window has no cursor");
        let line_drawer = get_line_drawer(config);
        line_drawer.draw_line_numbers(area, document, window, cursor, buffer, config);
    }

    pub fn draw_statusline(&self, buffer: &mut Buffer, area: Rect, config: GlyphConfig) {
        let statusline = &config.statusline;

        let clear = " ".repeat(area.width as usize);
        buffer.set_string(area.x, area.y, clear, HighlightGroup::default());

        let left_sections: Vec<StatusSection> = statusline
            .left
            .iter()
            .map(|section| {
                let content = match &section.content {
                    StatuslineContent::Immediate(inner) => inner.to_owned(),
                    StatuslineContent::Dynamic(fun) => fun
                        .call::<mlua::String>(())
                        .expect("failed to convert statusline to string")
                        .to_string_lossy(),
                };
                let style = match &section.style {
                    StatuslineStyle::HighlightGroup(group) => group,
                    StatuslineStyle::Named(group) => config
                        .highlight_groups
                        .get(group)
                        .expect("unregistered highlight_group"),
                };
                StatusSection::new(content, *style)
            })
            .collect();

        let right_sections: Vec<StatusSection> = statusline
            .right
            .iter()
            .map(|section| {
                let content = match &section.content {
                    StatuslineContent::Immediate(inner) => inner.to_owned(),
                    StatuslineContent::Dynamic(fun) => fun
                        .call::<mlua::String>(())
                        .expect("failed to convert statusline to string")
                        .to_string_lossy(),
                };
                let style = match &section.style {
                    StatuslineStyle::HighlightGroup(group) => group,
                    StatuslineStyle::Named(group) => config
                        .highlight_groups
                        .get(group)
                        .expect("using unregistered highlight_group"),
                };
                StatusSection::new(content, *style)
            })
            .collect();

        let mut current_x = area.x;
        for section in left_sections {
            buffer.set_string(current_x, area.y, &section.content, section.style);
            current_x += section.width as u16;
        }

        let mut current_x = area.x + area.width;
        for section in right_sections.into_iter().rev() {
            current_x -= section.width as u16;
            buffer.set_string(current_x, area.y, &section.content, section.style);
        }
    }

    pub fn draw_commandline(&self, buffer: &mut Buffer, ctx: &mut Context, area: Rect, config: GlyphConfig) {
        let clear = " ".repeat(area.width as usize);
        buffer.set_string(area.x, area.y, clear, HighlightGroup::default());

        let editor = ctx.editor.read();
        let message = match editor.mode() {
            // when on normal mode, we display the last message
            Mode::Normal => {
                let keys = editor.buffered_keymap.to_string();
                let message = editor.messages.to_string();
                let padding = area.width - 10 - message.len() as u16;
                let padding = " ".repeat(padding as usize);
                format!("{message}{padding}{keys}")
            }
            // when on insert mode, show buffered keymaps
            Mode::Insert => {
                let keys = editor.buffered_keymap.to_string();
                let padding = area.width - 10;
                format!("{}{keys}", " ".repeat(padding as usize))
            }
            // when on command mode, show the command
            Mode::Command => format!(":{}", editor.command),
        };
        let style = config.highlight_groups.get("foreground").copied().unwrap_or_default();
        buffer.set_string(area.x, area.y, message, style);
    }

    pub fn handle_key_event(
        &self,
        key_event: &KeyEvent,
        ctx: &mut Context,
        config: GlyphConfig,
    ) -> Result<Option<EventResult>, std::io::Error> {
        let mode = ctx.editor.read().mode();
        let mut editor = ctx.editor.write();
        editor.messages.clear();

        let keymap = format!("{}{}", editor.buffered_keymap, stringify_key(key_event));
        let result = config
            .keymaps
            .get(&mode)
            .expect("should have keymaps for all modes")
            .find_word(&keymap)
            .map(KeymapResult::from_query_result)
            .unwrap_or_default();

        match result.action {
            // if there is no keymap we do nothing, as behavior varies by mode
            KeymapAction::None if mode == Mode::Normal => editor.buffered_keymap.clear(),
            KeymapAction::None => {}
            KeymapAction::Continue => {
                editor.buffered_keymap.push_str(&keymap);
                return Ok(Some(EventResult::Consumed(None)));
            }
            KeymapAction::Execute => {
                let keymap = result.keymap.expect("tried to execute non-existing keymap");
                editor.buffered_keymap.clear();
                editor.command.clear();
                drop(editor);
                let mut context = CmdContext {
                    editor: ctx.editor.clone(),
                    cursors: ctx.cursors.clone(),
                    highlighter: ctx.highlighter,
                };
                keymap.command.run(&mut context);
                return Ok(Some(EventResult::Consumed(None)));
            }
        }

        match mode {
            // in normal mode everything is a keymap, if not handled, there is nothing to do here
            Mode::Normal => {}
            Mode::Insert => match key_event.code {
                KeyCode::Char(ch) => {
                    let mut context = CmdContext {
                        editor: ctx.editor.clone(),
                        cursors: ctx.cursors.clone(),
                        highlighter: ctx.highlighter,
                    };
                    drop(editor);
                    glyph_core::command::insert_char(&mut context, ch)
                }
                KeyCode::Backspace => {
                    let mut context = CmdContext {
                        editor: ctx.editor.clone(),
                        cursors: ctx.cursors.clone(),
                        highlighter: ctx.highlighter,
                    };
                    drop(editor);
                    glyph_core::command::remove_prev_char_breaking(&mut context)
                }
                KeyCode::Enter => {
                    let mut context = CmdContext {
                        editor: ctx.editor.clone(),
                        cursors: ctx.cursors.clone(),
                        highlighter: ctx.highlighter,
                    };
                    drop(editor);
                    glyph_core::command::break_line(&mut context)
                }
                _ => {}
            },
            Mode::Command => match key_event.code {
                KeyCode::Char(ch) => editor.command.push(ch),
                KeyCode::Enter => {
                    if editor.command.is_empty() {
                        return Ok(None);
                    }

                    let pieces = editor.command.split_whitespace().collect::<Vec<_>>();

                    if let Some(command) = config.user_commands.get(pieces[0]) {
                        command.call::<()>(&pieces[1..]).expect("TODO: HANDLE ERROR");
                        editor.command.clear();
                        editor.set_mode(Mode::Normal);
                    }
                }
                KeyCode::Backspace => {
                    if editor.command.is_empty() {
                        editor.set_mode(Mode::Normal);
                    }
                    editor.command.pop();
                }
                _ => {}
            },
        }

        Ok(None)
    }

    fn handle_resize(&self, new_area: Rect, ctx: &mut Context) -> Result<Option<EventResult>, std::io::Error> {
        ctx.editor.write().resize(new_area);
        Ok(None)
    }
}

impl RenderLayer for EditorLayer {
    fn draw(&self, buffer: &mut Buffer, ctx: &mut Context, config: GlyphConfig) {
        let mut area = ctx.editor.read().area();
        let commandline_area = area.split_bottom(1);
        let statusline_area = area.split_bottom(1);

        let style = config.highlight_groups.get("background").copied().unwrap_or_default();
        buffer.set_range_style(CellRange::<Point>::all(), StyleDef::replace(style));

        self.draw_statusline(buffer, statusline_area, config);
        self.draw_commandline(buffer, ctx, commandline_area, config);

        let editor = ctx.editor.read();
        for window in editor.focused_tab().tree.windows().values() {
            let document = editor.document(window.document);
            self.draw_window(document, window, buffer, ctx.highlighter, ctx.cursors.clone(), config);
        }
    }

    fn cursor(&self, ctx: &mut Context, config: GlyphConfig) -> (Option<Point>, CursorKind) {
        let editor = ctx.editor.read();

        match editor.mode() {
            Mode::Normal | Mode::Insert => {
                let tab = editor.focused_tab();
                let focused_window = tab.tree.focus();
                let window = tab.tree.window(focused_window);
                let cursors = ctx.cursors.read();
                let cursor = cursors.get(&window.id).expect("window has no cursor");
                let document = editor.document(window.document);
                let gutter_size = calculate_gutter_size(document, config);

                let point = Point {
                    x: window.area.x + ((cursor.x() + gutter_size as usize) - window.scroll().0) as u16,
                    y: window.area.y + (cursor.y() - window.scroll().1) as u16,
                };

                (Some(point), CursorKind::Block)
            }
            Mode::Command => {
                let command = editor.command.len() as u16;
                let mut area = editor.area();
                let area = area.split_bottom(1);

                let point = Point {
                    x: area.x + command + 1,
                    y: area.y,
                };
                (Some(point), CursorKind::Block)
            }
        }
    }

    fn handle_event(
        &self,
        event: &Event,
        ctx: &mut Context,
        config: GlyphConfig,
    ) -> Result<Option<EventResult>, std::io::Error> {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event, ctx, config),
            Event::Resize(width, height) => self.handle_resize((height, width).into(), ctx),
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

fn stringify_key(key_event: &KeyEvent) -> String {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Char(ch), KeyModifiers::CONTROL) => format!("<c-{ch}>"),
        (KeyCode::Char(ch), _) => ch.to_string(),
        (KeyCode::Enter, _) => String::from("<cr>"),
        (KeyCode::Backspace, _) => String::from("<bs>"),
        (KeyCode::Esc, _) => String::from("<esc>"),
        (KeyCode::F(num), _) => format!("<f-{num}>"),
        _ => String::default(),
    }
}
