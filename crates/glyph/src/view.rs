use crossterm::cursor;
use crossterm::style::{Color, Print, Stylize};
use crossterm::{terminal, QueueableCommand};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{stdout, Result, Stdout, Write};
use std::rc::Rc;

use crate::buffer::Buffer;
use crate::command::{Command, EditorCommands};
use crate::config::Config;
use crate::pane::{Pane, PaneDimensions, Position};
use crate::window::Window;

#[derive(Default, Debug, Copy, Clone)]
pub struct ViewSize {
    pub height: u16,
    pub width: u16,
}

impl From<(u16, u16)> for ViewSize {
    fn from((width, height): (u16, u16)) -> Self {
        Self { width, height }
    }
}

pub struct View {
    active_window: Rc<RefCell<Window>>,
    size: ViewSize,
    stdout: Stdout,
    config: &'static Config,
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
        })
    }

    pub fn handle(&mut self, command: Command) -> Result<()> {
        match command {
            Command::Editor(EditorCommands::Start) => self.initialize()?,
            Command::Editor(EditorCommands::Quit) => self.shutdown()?,
            Command::Editor(EditorCommands::SecondElapsed) => self.draw_statusbar()?,
            Command::Buffer(_) => self.handle_buffer(command)?,
            Command::Cursor(_) => self.handle_cursor(command)?,
            Command::Pane(_) => self.active_window.borrow_mut().handle(command)?,
            Command::Window(_) => self.active_window.borrow_mut().handle(command)?,
        };
        Ok(())
    }

    fn handle_cursor(&mut self, command: Command) -> Result<()> {
        self.active_window.borrow_mut().handle(command)?;
        self.draw_statusbar()?;
        Ok(())
    }

    fn handle_buffer(&mut self, command: Command) -> Result<()> {
        self.active_window.borrow_mut().handle(command)?;
        self.draw_statusbar()?;
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
        self.clear_screen()?;
        self.draw_statusbar()?;
        self.active_window.borrow_mut().initialize()?;
        Ok(())
    }

    fn clear_screen(&mut self) -> Result<()> {
        self.stdout
            .queue(cursor::MoveTo(0, 0))?
            .queue(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    fn draw_statusbar(&mut self) -> Result<()> {
        self.stdout
            .queue(cursor::SavePosition)?
            .queue(cursor::Hide)?;

        self.draw_statusbar_background()?;

        let active_pane = self.active_window.borrow_mut().get_active_pane();
        let cursor_position = active_pane.borrow().get_cursor_readable_position();
        let Position { col, row } = cursor_position;
        let lines = active_pane.borrow().get_buffer().borrow().marker.len() as u16;

        let cursor = format!("{}:{}", row, col);
        let percentage = match row {
            1 => "TOP".into(),
            _ if row == lines => "BOTTOM".into(),
            _ => format!("{}%", (row as f64 / lines as f64 * 100.0) as usize),
        };
        let file_name = active_pane.borrow().get_buffer().borrow().file_name.clone();
        let file_name = file_name.split('/').rev().nth(0).unwrap();
        let file_name = format!("\u{eae9} {}", file_name);

        let cursor_pad = self.size.width - cursor.len() as u16 - self.config.sidebar_width;
        let percentage_pad = cursor_pad - 2 - percentage.len() as u16;
        let filename_pad = 2;

        self.stdout
            .queue(cursor::MoveTo(cursor_pad as u16, self.size.height))?
            .queue(Print(cursor.with(Color::Green)))?
            .queue(cursor::MoveTo(percentage_pad as u16, self.size.height))?
            .queue(Print(percentage.with(Color::Magenta)))?
            .queue(cursor::MoveTo(filename_pad as u16, self.size.height))?
            .queue(Print(file_name.with(Color::White)))?
            .queue(cursor::RestorePosition)?
            .queue(cursor::Show)?;

        Ok(())
    }

    fn draw_statusbar_background(&mut self) -> Result<()> {
        for col in 0..self.size.width {
            self.stdout
                .queue(cursor::MoveTo(col, self.size.height))?
                .queue(Print(" ".to_string().with(Color::Black)))?;
        }
        Ok(())
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
