use std::{
    cell::RefCell,
    collections::HashMap,
    io::{stdout, Result, Stdout, Write},
    rc::Rc,
};

use crossterm::{cursor, terminal, QueueableCommand};

use crate::{buffer::Buffer, pane::Pane, window::Window};

#[derive(Default, Debug)]
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
    size: ViewSize,
    stdout: Stdout,
    next_pane_id: u16,
    next_buffer_id: u16,
    next_window_id: u16,
}

impl View {
    pub fn new() -> Self {
        Self {
            stdout: stdout(),
            windows: HashMap::new(),
            size: ViewSize::default(),
            next_buffer_id: 0,
            next_window_id: 0,
            next_pane_id: 0,
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.size = terminal::size()?.into();

        let buffer = self.make_buffer();
        let pane = self.make_pane(buffer);

        self.windows.insert(
            self.next_window_id,
            Rc::new(RefCell::new(Window::new(1, self.size, pane))),
        );

        self.next_pane_id += 1;
        self.next_buffer_id += 1;
        self.next_window_id += 1;
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.clear_screen();
        self.stdout.flush()?;
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        self.clear_screen()
    }

    fn clear_screen(&self) -> Result<()> {
        self.stdout
            .queue(cursor::MoveTo(0, 0))?
            .queue(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    fn make_buffer(&self) -> Rc<RefCell<Buffer>> {
        Rc::new(RefCell::new(Buffer::new(self.next_buffer_id, None)))
    }

    fn make_pane(&self, buffer: Rc<RefCell<Buffer>>) -> Rc<RefCell<Pane>> {
        Rc::new(RefCell::new(Pane::new(self.next_pane_id, buffer)))
    }
}
