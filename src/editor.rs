use crossterm::terminal;
use std::collections::HashMap;
use std::io::Result;
use std::sync::{Arc, Mutex};

use crate::buffer::Buffer;
use crate::event_handler::EventHandler;
use crate::pane::Pane;
use crate::window::Window;

pub struct Editor {
    pub event_handler: EventHandler,
    pub buffers: HashMap<u16, Arc<Mutex<Buffer>>>,
    pub panes: HashMap<u16, Arc<Mutex<Pane>>>,
    pub windows: HashMap<u16, Window>,
}

impl Editor {
    pub fn new() -> Self {
        let mut buffers = HashMap::new();
        let mut panes = HashMap::new();
        let mut windows = HashMap::new();

        let buffer = Arc::new(Mutex::new(Buffer::new()));
        let pane = Arc::new(Mutex::new(Pane::new(0, Arc::clone(&buffer))));
        buffers.insert(0, Arc::clone(&buffer));
        panes.insert(0, Arc::clone(&pane));
        windows.insert(0, Window::new(Arc::clone(&pane)));

        Self {
            buffers,
            event_handler: EventHandler::new(),
            panes,
            windows,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;

        while self.event_handler.is_quitting == false {
            println!("{:?}\r", self.panes);
            println!("{:?}\r", self.buffers);
            println!("{:?}\r", self.windows);
            self.event_handler.poll_events()?;
        }

        terminal::disable_raw_mode()?;
        Ok(())
    }
}
