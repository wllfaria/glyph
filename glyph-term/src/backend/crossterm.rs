use std::io::Write;

use crossterm::event::{DisableFocusChange, EnableFocusChange};
use crossterm::{cursor, execute, queue, style, terminal};
use glyph_config::{CursorConfig, CursorStyle, GlyphConfig};
use glyph_core::rect::Rect;

use super::{Backend, CursorKind, Drawable};
use crate::graphics::IntoColor;

#[derive(Debug)]
pub struct CrosstermBackend<W: Write> {
    buffer: W,
}

trait IntoCursorStyle {
    fn into_cursor_style(self) -> cursor::SetCursorStyle;
}

impl IntoCursorStyle for CursorConfig {
    fn into_cursor_style(self) -> cursor::SetCursorStyle {
        match self.style {
            CursorStyle::Block => cursor::SetCursorStyle::SteadyBlock,
        }
    }
}

impl<W> CrosstermBackend<W>
where
    W: Write,
{
    pub fn new(buffer: W) -> CrosstermBackend<W> {
        CrosstermBackend { buffer }
    }
}

impl<W> Backend for CrosstermBackend<W>
where
    W: Write,
{
    fn setup(&mut self) -> Result<(), std::io::Error> {
        terminal::enable_raw_mode()?;
        execute!(self.buffer, terminal::EnterAlternateScreen, EnableFocusChange)?;
        Ok(())
    }

    fn restore(&mut self) -> Result<(), std::io::Error> {
        execute!(self.buffer, DisableFocusChange, terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()
    }

    fn draw<'a, I, T>(&mut self, content: I, config: GlyphConfig) -> Result<(), std::io::Error>
    where
        I: Iterator<Item = T>,
        T: Into<Drawable<'a>>,
    {
        self.hide_cursor()?;

        for item in content.into_iter() {
            let drawable: Drawable<'a> = item.into();

            queue!(
                self.buffer,
                cursor::MoveTo(drawable.x, drawable.y),
                config.cursor().clone().into_cursor_style(),
                style::SetForegroundColor(drawable.cell.style.fg.into_color()),
                style::SetBackgroundColor(drawable.cell.style.bg.into_color()),
                style::Print(drawable.cell.symbol)
            )?;
        }
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<(), std::io::Error> {
        execute!(self.buffer, cursor::Hide)
    }

    fn show_cursor(&mut self) -> Result<(), std::io::Error> {
        execute!(self.buffer, cursor::Show)
    }

    fn set_cursor(&mut self, x: u16, y: u16, kind: CursorKind) -> Result<(), std::io::Error> {
        match kind {
            CursorKind::Block => execute!(
                self.buffer,
                cursor::SetCursorStyle::SteadyBlock,
                cursor::MoveTo(x, y),
                cursor::Show
            ),
            CursorKind::Hidden => self.hide_cursor(),
        }
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.buffer.flush()
    }

    fn area(&self) -> Result<Rect, std::io::Error> {
        let (width, height) = terminal::size()?;
        Ok(Rect::new(0, 0, width, height))
    }
}
