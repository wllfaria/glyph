use crossterm::{cursor, terminal, QueueableCommand};
use std::collections::HashMap;
use std::io::{Result, Write};

use crate::event_handler::EventHandler;

pub struct Buffer {}
pub struct Pane {}
pub struct Window {}

pub struct Editor {
    pub windows: HashMap<u16, Window>,
    pub buffers: HashMap<u16, Buffer>,
    pub panes: HashMap<u16, Pane>,
    pub event_handler: EventHandler,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
            windows: HashMap::new(),
            panes: HashMap::new(),
            event_handler: EventHandler::new(),
        }
    }

    pub fn start(&mut self) -> Result<()> {
        let mut stdout = std::io::stdout();
        terminal::enable_raw_mode()?;
        while self.event_handler.is_quitting == false {
            stdout
                .queue(terminal::Clear(terminal::ClearType::All))?
                .queue(cursor::MoveTo(0, 0))?
                .flush()?;
            self.event_handler.poll_events()?;
            stdout.flush()?;
        }
        terminal::disable_raw_mode()?;

        Ok(())
    }
}
