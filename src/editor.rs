use std::io::{stdout, Result, Write};

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
            match self.events.poll()? {
                Some(Command::Editor(EditorCommands::Quit)) => self.is_running = false,
                Some(command) => self.view.handle_command(command),
                _ => (),
            }
            self.view.render()?;
            stdout().flush()?;
        }

        self.view.shutdown()?;
        Ok(())
    }
}
