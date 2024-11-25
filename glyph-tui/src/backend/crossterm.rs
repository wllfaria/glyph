use std::io::Write;

use crossterm::event::{DisableFocusChange, EnableFocusChange};
use crossterm::{cursor, execute, terminal};

use super::Backend;

#[derive(Debug)]
pub struct CrosstermBackend<W: Write> {
    buffer: W,
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

    fn draw<'a, I, T>(&mut self, content: I)
    where
        I: Iterator<Item = T>,
        T: Into<super::Drawable<'a>>,
    {
        todo!()
    }

    fn hide_cursor(&mut self) -> Result<(), std::io::Error> {
        execute!(self.buffer, cursor::Hide)
    }

    fn show_cursor(&mut self) -> Result<(), std::io::Error> {
        execute!(self.buffer, cursor::Show)
    }

    fn set_cursor(&mut self, x: u16, y: u16) -> Result<(), std::io::Error> {
        execute!(self.buffer, cursor::MoveTo(x, y))
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.buffer.flush()
    }
}
