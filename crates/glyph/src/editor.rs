use std::io::{self, Write};

use crate::command::{Command, EditorCommands};
use crate::events::Events;
use crate::view::View;

pub struct Editor {
    is_running: bool,
    events: Events,
    view: View,
}

impl Editor {
    pub fn new(file_name: Option<String>) -> io::Result<Self> {
        Ok(Self {
            is_running: true,
            events: Events::new(),
            view: View::new(file_name)?,
        })
    }

    pub fn start(&mut self) -> io::Result<()> {
        self.view.handle(Command::Editor(EditorCommands::Start))?;

        while self.is_running {
            match self.events.poll()? {
                Some(Command::Editor(EditorCommands::Quit)) => {
                    self.view.handle(Command::Editor(EditorCommands::Quit))?;
                    self.is_running = false
                }
                Some(command) => self.view.handle(command)?,
                _ => (),
            }
            io::stdout().flush()?;
        }
        Ok(())
    }
}
