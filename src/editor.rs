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
use crate::state::State;
use crate::window::Window;

pub struct Editor {
    pub event_handler: Keyboard,
    pub buffers: HashMap<u16, Arc<Mutex<Buffer>>>,
    pub panes: HashMap<u16, Rc<RefCell<Pane>>>,
    pub windows: HashMap<u16, Rc<RefCell<Window>>>,
    pub state: Rc<RefCell<State>>,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let state = Rc::new(RefCell::new(State::new()));
        let commands = Commands::make_commands(state.clone());

        Ok(Self {
            state: state.clone(),
            panes: HashMap::new(),
            buffers: HashMap::new(),
            windows: HashMap::new(),
            event_handler: Keyboard::new(state.clone(), commands),
        })
    }

    pub fn populate_empty(&mut self, filename: Option<String>) -> Result<()> {
        let pane = Rc::new(RefCell::new(Pane::new(1, self.state.clone())));
        let window = Rc::new(RefCell::new(Window::new(1, self.state.clone())?));
        let buffer = Arc::new(Mutex::new(Buffer::new(1, filename, self.state.clone())));
        let mut state = self.state.borrow_mut();

        pane.borrow_mut().attach_buffer(buffer.clone());
        window.borrow_mut().attach_pane(pane.clone());

        self.panes.insert(1, pane.clone());
        self.windows.insert(1, window.clone());
        self.buffers.insert(1, buffer.clone());

        state.set_active_buffer(buffer.clone());
        state.set_active_pane(pane.clone());
        state.set_active_window(window.clone());
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;

        while !self.state.borrow().is_quitting {
            match self.state.borrow().active_window {
                Some(ref window) => window.borrow_mut().render()?,
                None => break,
            };
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
