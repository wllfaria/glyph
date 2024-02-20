use crossterm::style::Print;
use crossterm::{cursor, style};
use crossterm::{terminal, QueueableCommand};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{stdout, Result, Stdout, Write};
use std::rc::Rc;

use crate::buffer::Buffer;
use crate::command::{Command, EditorCommands};
use crate::config::Config;
use crate::pane::{Pane, PaneDimensions, Position};
use crate::theme::Theme;
use crate::viewport::{Change, Viewport};
use crate::window::Window;

#[derive(Default, Debug, Copy, Clone)]
pub struct Size {
    pub height: u16,
    pub width: u16,
}

impl From<(u16, u16)> for Size {
    fn from((width, height): (u16, u16)) -> Self {
        Self { width, height }
    }
}

pub struct View {
    active_window: Rc<RefCell<Window>>,
    size: Size,
    stdout: Stdout,
    config: &'static Config,
    viewport: Viewport,
    theme: &'static Theme,
}

impl View {
    pub fn new(file_name: Option<String>) -> Result<Self> {
        let mut windows = HashMap::new();
        let size = terminal::size()?;
        let mut window_size = size;
        window_size.1 -= 1;
        let buffer = View::make_buffer(1, file_name)?;
        let pane = View::make_pane(1, buffer, window_size.into());
        let window = View::make_window(1, pane);
        windows.insert(window.borrow().id, window.clone());

        Ok(Self {
            stdout: stdout(),
            size: size.into(),
            active_window: window.clone(),
            config: Config::get(),
            viewport: Viewport::new(size.0 as usize, 1),
            theme: Theme::get(),
        })
    }

    pub fn handle(&mut self, command: Command) -> Result<()> {
        let last_viewport = self.viewport.clone();
        let mut viewport = Viewport::new(self.size.width as usize, 1);
        match command {
            Command::Editor(EditorCommands::Start) => self.initialize()?,
            Command::Editor(EditorCommands::Quit) => self.shutdown()?,
            Command::Editor(EditorCommands::SecondElapsed) => self.draw_statusline(&mut viewport),
            Command::Buffer(_) => self.handle_buffer(command, &mut viewport)?,
            Command::Cursor(_) => self.handle_cursor(command, &mut viewport)?,
            Command::Pane(_) => self.active_window.borrow_mut().handle(command)?,
            Command::Window(_) => self.active_window.borrow_mut().handle(command)?,
        };
        self.stdout.queue(cursor::SavePosition)?;
        self.render_statusline(viewport.diff(&last_viewport))?;
        self.viewport = viewport;
        self.stdout.queue(cursor::RestorePosition)?.flush()?;
        
        Ok(())
    }

    fn handle_cursor(&mut self, command: Command, viewport: &mut Viewport) -> Result<()> {
        self.active_window.borrow_mut().handle(command)?;
        self.draw_statusline(viewport);
        Ok(())
    }

    fn handle_buffer(&mut self, command: Command, viewport: &mut Viewport) -> Result<()> {
        self.active_window.borrow_mut().handle(command)?;
        self.draw_statusline(viewport);
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        self.clear_screen()?;
        self.stdout.queue(terminal::LeaveAlternateScreen)?.flush()?;
        terminal::disable_raw_mode()?;

        Ok(())
    }

    fn initialize(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        self.stdout.queue(terminal::EnterAlternateScreen)?;

        let last_viewport = self.viewport.clone();
        let mut viewport = Viewport::new(self.size.width as usize, 1);
        self.clear_screen()?;
        self.draw_statusline(&mut viewport);
        self.render_statusline(viewport.diff(&last_viewport))?;

        self.active_window.borrow_mut().initialize()?;
        self.viewport = viewport;

        Ok(())
    }

    fn render_statusline(&mut self, changes: Vec<Change>) -> Result<()> {
        for change in changes {
            self.stdout
                .queue(cursor::MoveTo(change.col as u16, self.size.height))?;

            if let Some(bg) = change.cell.style.bg {
                self.stdout.queue(style::SetBackgroundColor(bg))?;
            } else {
                self.stdout
                    .queue(style::SetBackgroundColor(self.theme.style.bg.unwrap()))?;
            }
            if let Some(fg) = change.cell.style.fg {
                self.stdout.queue(style::SetForegroundColor(fg))?;
            } else {
                self.stdout
                    .queue(style::SetForegroundColor(self.theme.style.fg.unwrap()))?;
            }

            self.stdout.queue(Print(change.cell.c))?;
        }
        Ok(())
    }

    fn clear_screen(&mut self) -> Result<()> {
        self.stdout
            .queue(cursor::MoveTo(0, 0))?
            .queue(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    fn draw_statusline(&mut self, viewport: &mut Viewport) {
        let active_pane = self.active_window.borrow_mut().get_active_pane();
        let cursor_position = active_pane.borrow().get_cursor_readable_position();
        let Position { col, row } = cursor_position;
        let lines = active_pane.borrow().get_buffer().borrow().marker.len() as u16;

        let cursor = format!("{}:{} ", row, col);
        let percentage = match row {
            1 => "TOP ".into(),
            _ if row == lines => "BOT ".into(),
            _ => format!("{}% ", (row as f64 / lines as f64 * 100.0) as usize),
        };

        let file_name = active_pane.borrow().get_buffer().borrow().file_name.clone();
        let file_name = file_name.split('/').rev().nth(0).unwrap();
        let file_name = format!(" {}", file_name);

        let padding = " "
            .repeat(self.size.width as usize - file_name.len() - cursor.len() - percentage.len());

        viewport.set_text(0, 0, &file_name, &self.theme.statusline.inner);
        viewport.set_text(file_name.len(), 0, &padding, &self.theme.statusline.inner);

        viewport.set_text(
            self.size.width as usize - 1 - cursor.len(),
            0,
            &cursor,
            &self.theme.statusline.inner,
        );

        viewport.set_text(
            self.size.width as usize - cursor.len(),
            0,
            &cursor,
            &self.theme.statusline.inner,
        );

        viewport.set_text(
            self.size.width as usize - cursor.len() - percentage.len(),
            0,
            &percentage,
            &self.theme.statusline.inner,
        );
    }

    fn make_buffer(id: u16, file_name: Option<String>) -> Result<Rc<RefCell<Buffer>>> {
        let buffer = Buffer::new(id, file_name)?;
        Ok(Rc::new(RefCell::new(buffer)))
    }

    fn make_pane(
        id: u16,
        buffer: Rc<RefCell<Buffer>>,
        dimensions: PaneDimensions,
    ) -> Rc<RefCell<Pane>> {
        Rc::new(RefCell::new(Pane::new(id, buffer, dimensions)))
    }

    fn make_window(id: u16, pane: Rc<RefCell<Pane>>) -> Rc<RefCell<Window>> {
        Rc::new(RefCell::new(Window::new(id, pane)))
    }
}
