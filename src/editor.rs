use crossterm::terminal;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Result;
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

#[derive(Debug, PartialEq)]
pub enum EditorModes {
    Normal,
    Insert,
    Command,
}

impl Editor {
    pub fn new(filename: Option<String>) -> Result<Self> {
        let mut buffers = HashMap::new();
        let mut panes = HashMap::new();
        let mut windows = HashMap::new();

        let buffer = Arc::new(Mutex::new(Buffer::new(filename)));
        let pane = Rc::new(RefCell::new(Pane::new(0, Arc::clone(&buffer))));
        let window = Rc::new(RefCell::new(Window::new(Rc::clone(&pane))?));
        let state = Rc::new(RefCell::new(State::new(
            Arc::clone(&buffer),
            Rc::clone(&pane),
            Rc::clone(&window),
        )));
        let commands = Commands::make_commands(Rc::clone(&state));

        buffers.insert(0, Arc::clone(&buffer));
        panes.insert(0, Rc::clone(&pane));
        windows.insert(0, Rc::clone(&window));

        Ok(Self {
            buffers,
            event_handler: Keyboard::new(Rc::clone(&state), commands),
            panes,
            windows,
            state,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;

        while !self.state.borrow().is_quitting {
            self.state.borrow().active_window.borrow_mut().render()?;
            self.event_handler.poll_events()?;
        }

        terminal::disable_raw_mode()?;
        Ok(())
    }
}
