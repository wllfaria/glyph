use crossterm::style::Print;
use crossterm::{cursor, style};
use crossterm::{terminal, QueueableCommand};
use std::collections::HashMap;
use std::io::{stdout, Result, Stdout, Write};

use crate::command::{Command, EditorCommands};
use crate::config::Config;
use crate::lsp::LspClient;
use crate::pane::Position;
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

pub struct View<'a> {
    active_window: usize,
    windows: HashMap<usize, Window<'a>>,
    size: Size,
    stdout: Stdout,
    config: &'static Config,
    viewport: Viewport,
    theme: &'static Theme,
    lsp: &'a LspClient,
}

impl<'a> View<'a> {
    pub fn new(lsp: &'a LspClient, mut window: Window<'a>) -> Result<Self> {
        let mut windows = HashMap::new();
        let size = terminal::size()?;

        let id = window.id;
        window.resize((size.0, size.1 - 1).into());
        windows.insert(window.id, window);

        Ok(Self {
            stdout: stdout(),
            size: size.into(),
            active_window: id,
            windows,
            config: Config::get(),
            viewport: Viewport::new(size.0 as usize, 1),
            theme: Theme::get(),
            lsp,
        })
    }

    pub fn handle(&mut self, command: Command) -> Result<()> {
        let last_viewport = self.viewport.clone();
        let mut viewport = Viewport::new(self.size.width as usize, 1);
        let active_window = self.windows.get_mut(&self.active_window).unwrap();
        match command {
            Command::Editor(EditorCommands::Start) => self.initialize()?,
            Command::Editor(EditorCommands::Quit) => self.shutdown()?,
            Command::Editor(EditorCommands::SecondElapsed) => self.draw_statusline(&mut viewport),
            Command::Buffer(_) => self.handle_buffer(command, &mut viewport)?,
            Command::Cursor(_) => self.handle_cursor(command, &mut viewport)?,
            Command::Pane(_) => active_window.handle(command)?,
            Command::Window(_) => active_window.handle(command)?,
        };
        self.stdout.queue(cursor::SavePosition)?;
        self.render_statusline(viewport.diff(&last_viewport))?;
        self.viewport = viewport;
        self.stdout.queue(cursor::RestorePosition)?.flush()?;

        Ok(())
    }

    fn get_active_window(&self) -> &Window {
        self.windows.get(&self.active_window).unwrap()
    }

    fn handle_cursor(&mut self, command: Command, viewport: &mut Viewport) -> Result<()> {
        let active_window = self.windows.get_mut(&self.active_window).unwrap();
        active_window.handle(command)?;
        self.draw_statusline(viewport);
        Ok(())
    }

    fn handle_buffer(&mut self, command: Command, viewport: &mut Viewport) -> Result<()> {
        let active_window = self.windows.get_mut(&self.active_window).unwrap();
        active_window.handle(command)?;
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

        self.windows
            .get_mut(&self.active_window)
            .unwrap()
            .initialize()?;
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
        let active_pane = self.get_active_window().get_active_pane();
        let cursor_position = active_pane.get_cursor_readable_position();
        let Position { col, row } = cursor_position;
        let lines = active_pane.get_buffer().borrow().marker.len();

        let cursor = format!("{}:{} ", row, col);
        let percentage = match row {
            1 => "TOP ".into(),
            _ if row == lines => "BOT ".into(),
            _ => format!("{}% ", (row as f64 / lines as f64 * 100.0) as usize),
        };

        let file_name = active_pane.get_buffer().borrow().file_name.clone();
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
}
