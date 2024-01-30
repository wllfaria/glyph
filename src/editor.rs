use crossterm::{terminal, QueueableCommand};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Result, Write};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::buffer::Buffer;
use crate::commands::Commands;
use crate::keyboard::Keyboard;
use crate::pane::Pane;
use crate::window::Window;

pub struct Editor {
    pub event_handler: Keyboard,
    pub buffers: HashMap<u16, Arc<Mutex<Buffer>>>,
    pub panes: HashMap<u16, Rc<RefCell<Pane>>>,
    pub windows: HashMap<u16, Rc<RefCell<Window>>>,
    is_running: bool,
}

impl Editor {
    pub fn new() -> Self {
        let commands = Commands::make_commands();

        Self {
            panes: HashMap::new(),
            buffers: HashMap::new(),
            windows: HashMap::new(),
            event_handler: Keyboard::new(commands),
            is_running: true,
        }
    }

    pub fn populate_empty(&mut self, filename: Option<String>) -> Result<()> {
        let pane = Rc::new(RefCell::new(Pane::new(1)));
        let window = Rc::new(RefCell::new(Window::new(1)?));
        let buffer = Arc::new(Mutex::new(Buffer::new(1, filename)));

        pane.borrow_mut().attach_buffer(buffer.clone());
        window.borrow_mut().attach_pane(pane.clone());

        self.panes.insert(1, pane.clone());
        self.windows.insert(1, window.clone());
        self.buffers.insert(1, buffer.clone());

        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;

        while self.is_running {
            self.event_handler.poll_events()?;
        }

        std::io::stdout().queue(crossterm::cursor::MoveTo(0, 0))?;
        for window in self.windows.values() {
            window.borrow_mut().clear()?;
        }

        std::io::stdout().flush()?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}
