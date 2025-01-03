use glyph_core::config::{GlyphConfig, LineNumbersConfig};
use glyph_core::cursor::Cursor;
use glyph_core::document::Document;
use glyph_core::rect::Rect;
use glyph_core::window::Window;

use crate::buffer::Buffer;
use crate::layers::editor_layer::digits_in_number;

#[derive(Debug)]
pub enum LineDrawer {
    Absolute(AbsoluteLineDrawer),
    Relative(RelativeLineDrawer),
    RelativeNumbered(RelativeNumberedLineDrawer),
}

pub trait LineNumberDrawer {
    fn draw_line_numbers(
        &self,
        area: Rect,
        document: &Document,
        window: &Window,
        cursor: &Cursor,
        buffer: &mut Buffer,
        config: GlyphConfig<'_>,
    );
}

#[derive(Debug, Default)]
pub struct AbsoluteLineDrawer;

#[derive(Debug, Default)]
pub struct RelativeLineDrawer;

#[derive(Debug, Default)]
pub struct RelativeNumberedLineDrawer;

impl LineNumberDrawer for AbsoluteLineDrawer {
    fn draw_line_numbers(
        &self,
        area: Rect,
        document: &Document,
        window: &Window,
        cursor: &Cursor,
        buffer: &mut Buffer,
        config: GlyphConfig<'_>,
    ) {
        let height = area.height as usize;
        let total_lines = document.text().len_lines();
        let start = window.scroll().y;
        let end = total_lines.min(start + height);
        let line_size = usize::max(digits_in_number(total_lines) + 1, 3);
        let x = area.x + config.gutter.sign_column.size();
        let mut line_str = String::with_capacity(line_size);

        let style = config
            .highlight_groups
            .get("line_number")
            .expect("unregistered highlight group for line_number");
        let current_line_style = config
            .highlight_groups
            .get("current_line")
            .expect("unregistered highlight group for current_line");

        for (row, line) in (start..end).enumerate() {
            line_str.clear();
            use std::fmt::Write;
            write!(&mut line_str, "{:>width$} ", line + 1, width = line_size).expect("failed to write to string");

            let diff = cursor.y().abs_diff(line);
            let style = if diff == 0 { current_line_style } else { style };
            buffer.set_string(x, area.y + row as u16, &line_str, *style);
        }

        let remaining = area.height as usize - (end - start);
        for line in 0..remaining {
            line_str.clear();
            use std::fmt::Write;

            let row = end - start + line;
            write!(&mut line_str, "{diff:>width$} ", diff = "~", width = line_size).expect("failed to write to string");
            buffer.set_string(x, area.y + row as u16, &line_str, *style);
        }
    }
}

impl LineNumberDrawer for RelativeLineDrawer {
    fn draw_line_numbers(
        &self,
        area: Rect,
        document: &Document,
        window: &Window,
        cursor: &Cursor,
        buffer: &mut Buffer,
        config: GlyphConfig<'_>,
    ) {
        let height = area.height as usize;
        let total_lines = document.text().len_lines();
        let start = window.scroll().y;
        let end = total_lines.min(start + height);
        let line_size = usize::max(digits_in_number(total_lines) + 1, 3);
        let x = area.x + config.gutter.sign_column.size();
        let mut line_str = String::with_capacity(line_size);

        let style = config
            .highlight_groups
            .get("line_number")
            .expect("unregistered highlight group for line_number");
        let current_line_style = config
            .highlight_groups
            .get("current_line")
            .expect("unregistered highlight group for current_line");

        for (row, line) in (start..end).enumerate() {
            line_str.clear();
            use std::fmt::Write;

            let diff = cursor.y().abs_diff(line);
            let style = if diff == 0 { current_line_style } else { style };

            write!(&mut line_str, "{:>width$} ", diff, width = line_size).expect("failed to write to string");
            buffer.set_string(x, area.y + row as u16, &line_str, *style);
        }

        let remaining = area.height as usize - (end - start);
        for line in 0..remaining {
            line_str.clear();
            use std::fmt::Write;
            let row = end - start + line;
            write!(&mut line_str, "{diff:>width$} ", diff = "~", width = line_size).expect("failed to write to string");
            buffer.set_string(x, area.y + row as u16, &line_str, *style);
        }
    }
}

impl LineNumberDrawer for RelativeNumberedLineDrawer {
    fn draw_line_numbers(
        &self,
        area: Rect,
        document: &Document,
        window: &Window,
        cursor: &Cursor,
        buffer: &mut Buffer,
        config: GlyphConfig<'_>,
    ) {
        let height = area.height as usize;
        let total_lines = document.text().len_lines();
        let start = window.scroll().y;
        let end = total_lines.min(start + height);
        let line_size = usize::max(digits_in_number(total_lines) + 1, 3);
        let x = area.x + config.gutter.sign_column.size();
        let mut line_str = String::with_capacity(line_size);

        let style = config
            .highlight_groups
            .get("line_number")
            .expect("unregistered highlight group for line_number");
        let current_line_style = config
            .highlight_groups
            .get("current_line")
            .expect("unregistered highlight group for current_line");

        for (row, line) in (start..end).enumerate() {
            line_str.clear();
            use std::fmt::Write;

            let diff = cursor.y().abs_diff(line);
            let style = if diff == 0 { current_line_style } else { style };
            let diff = if diff == 0 { cursor.y() + 1 } else { diff };

            write!(&mut line_str, "{:>width$} ", diff, width = line_size).expect("failed to write to string");
            buffer.set_string(x, area.y + row as u16, &line_str, *style);
        }

        let remaining = area.height as usize - (end - start);
        for line in 0..remaining {
            line_str.clear();
            use std::fmt::Write;
            let row = end - start + line;
            write!(&mut line_str, "{diff:>width$} ", diff = "~", width = line_size).expect("failed to write to string");
            buffer.set_string(x, area.y + row as u16, &line_str, *style);
        }
    }
}

impl LineNumberDrawer for LineDrawer {
    fn draw_line_numbers(
        &self,
        area: Rect,
        document: &Document,
        window: &Window,
        cursor: &Cursor,
        buffer: &mut Buffer,
        config: GlyphConfig<'_>,
    ) {
        match self {
            LineDrawer::Absolute(inner) => inner.draw_line_numbers(area, document, window, cursor, buffer, config),
            LineDrawer::Relative(inner) => inner.draw_line_numbers(area, document, window, cursor, buffer, config),
            LineDrawer::RelativeNumbered(inner) => {
                inner.draw_line_numbers(area, document, window, cursor, buffer, config)
            }
        }
    }
}

pub fn get_line_drawer(config: GlyphConfig<'_>) -> LineDrawer {
    match config.gutter.line_numbers {
        LineNumbersConfig::Absolute => LineDrawer::Absolute(AbsoluteLineDrawer),
        LineNumbersConfig::Relative => LineDrawer::Relative(RelativeLineDrawer),
        LineNumbersConfig::RelativeNumbered => LineDrawer::RelativeNumbered(RelativeNumberedLineDrawer),
    }
}
