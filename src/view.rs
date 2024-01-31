use crossterm::{
    cursor::{self, SetCursorStyle},
    style::{Color, Print, Stylize},
    terminal, QueueableCommand,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{stdout, Result, Stdout, Write},
    rc::Rc,
};

use crate::{
    buffer::Buffer,
    command::{Command, EditorCommands},
    pane::{Pane, PaneDimensions},
    window::Window,
};

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
    windows: HashMap<u16, Rc<RefCell<Window>>>,
    active_window: Rc<RefCell<Window>>,
    size: ViewSize,
    stdout: Stdout,
    next_pane_id: u16,
    next_buffer_id: u16,
    next_window_id: u16,
}

impl View {
    pub fn new(file_name: Option<String>) -> Result<Self> {
        let mut windows = HashMap::new();
        let size = terminal::size()?;
        let mut window_size = size.clone();
        window_size.1 -= 1;
        let buffer = View::make_buffer(1, file_name);
        let pane = View::make_pane(1, buffer, window_size.into());
        let window = View::make_window(1, window_size.into(), pane);
        windows.insert(window.borrow().id, window.clone());

        Ok(Self {
            windows,
            next_pane_id: 2,
            next_buffer_id: 2,
            next_window_id: 2,
            stdout: stdout(),
            size: size.into(),
            active_window: window.clone(),
        })
    }

    pub fn handle(&mut self, command: Command) -> Result<()> {
        match command {
            Command::Editor(EditorCommands::Start) => self.initialize()?,
            Command::Editor(EditorCommands::Quit) => self.shutdown()?,
            Command::Buffer(_) => self.active_window.borrow_mut().handle(command)?,
            Command::Cursor(_) => self.handle_cursor(command)?,
            Command::Pane(_) => self.active_window.borrow_mut().handle(command)?,
            Command::Window(_) => self.active_window.borrow_mut().handle(command)?,
            _ => (),
        };
        Ok(())
    }

    fn handle_cursor(&mut self, command: Command) -> Result<()> {
        self.active_window.borrow_mut().handle(command)?;
        self.stdout.queue(cursor::SavePosition)?;
        self.draw_statusbar()?;
        self.stdout.queue(cursor::RestorePosition)?;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        self.clear_screen()?;
        self.stdout.flush()?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn initialize(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
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
        self.draw_statusbar_background()?;
        let active_pane = self.active_window.borrow_mut().get_active_pane();
        let cursor_position = active_pane.borrow().get_cursor_readable_position();
        let (col, row) = cursor_position;
        let cursor = format!("{}:{}", row, col);
        let padding = self.size.width - cursor.len() as u16 - active_pane.borrow().sidebar_width;
        self.stdout
            .queue(cursor::MoveTo(padding as u16, self.size.height))?
            .queue(Print(cursor.with(Color::White)))?;
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

    fn make_buffer(id: u16, file_name: Option<String>) -> Rc<RefCell<Buffer>> {
        Rc::new(RefCell::new(Buffer::new(id, file_name)))
    }

    fn make_pane(
        id: u16,
        buffer: Rc<RefCell<Buffer>>,
        dimensions: PaneDimensions,
    ) -> Rc<RefCell<Pane>> {
        Rc::new(RefCell::new(Pane::new(id, buffer, dimensions)))
    }

    fn make_window(id: u16, size: ViewSize, pane: Rc<RefCell<Pane>>) -> Rc<RefCell<Window>> {
        Rc::new(RefCell::new(Window::new(id, size, pane)))
    }
}
