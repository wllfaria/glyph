use std::{
    cell::RefCell,
    collections::HashMap,
    io::{stdout, Result, Stdout, Write},
    rc::Rc,
};

use crossterm::{cursor, terminal, QueueableCommand};

use crate::{buffer::Buffer, command::Command, pane::Pane, window::Window};

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
        let size = terminal::size()?.into();
        let buffer = View::make_buffer(1, file_name);
        let pane = View::make_pane(1, buffer);
        let window = View::make_window(1, size, pane);
        windows.insert(window.borrow().id, window.clone());

        Ok(Self {
            windows,
            next_pane_id: 2,
            next_buffer_id: 2,
            next_window_id: 2,
            stdout: stdout(),
            size: ViewSize::default(),
            active_window: window.clone(),
        })
    }

    pub fn initialize(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.clear_screen()?;
        self.stdout.flush()?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        self.clear_screen()
    }

    pub fn handle_command(&self, command: Command) {
        match command {
            Command::Pane(_) => self.active_window.borrow_mut().handle_command(command),
            Command::Window(_) => self.active_window.borrow_mut().handle_command(command),
            _ => {}
        }
    }

    fn clear_screen(&mut self) -> Result<()> {
        self.stdout
            .queue(cursor::MoveTo(0, 0))?
            .queue(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    fn make_buffer(id: u16, file_name: Option<String>) -> Rc<RefCell<Buffer>> {
        Rc::new(RefCell::new(Buffer::new(id, file_name)))
    }

    fn make_pane(id: u16, buffer: Rc<RefCell<Buffer>>) -> Rc<RefCell<Pane>> {
        Rc::new(RefCell::new(Pane::new(id, buffer)))
    }

    fn make_window(id: u16, size: ViewSize, pane: Rc<RefCell<Pane>>) -> Rc<RefCell<Window>> {
        Rc::new(RefCell::new(Window::new(id, size, pane)))
    }
}
