use std::io::Result;

use crate::{
    command::{Command, EditorCommands},
    events::Events,
    view::View,
};

pub struct Editor {
    is_running: bool,
    events: Events,
    view: View,
}

impl Editor {
    pub fn new(file_name: Option<String>) -> Result<Self> {
        Ok(Self {
            is_running: true,
            events: Events::new(),
            view: View::new(file_name)?,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        self.view.initialize()?;
        while self.is_running {
            if let Some(command) = self.events.poll()? {
                match command {
                    Command::Editor(EditorCommands::Quit) => self.is_running = false,
                    _ => self.view.handle_command(command),
                }
            }
            self.view.render()?;
        }
        self.view.shutdown()?;
        Ok(())
    }
}
