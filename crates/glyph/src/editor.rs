use std::io::{self, Write};

use chrono::{DateTime, Duration, Utc};

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
        let mut last_time: DateTime<Utc> = Utc::now();

        while self.is_running {
            match self.events.poll()? {
                Some(Command::Editor(EditorCommands::Quit)) => {
                    self.view.handle(Command::Editor(EditorCommands::Quit))?;
                    self.is_running = false
                }
                Some(command) => self.view.handle(command)?,
                _ => (),
            }
            if self.has_second_passed(last_time) {
                last_time = Utc::now();
                self.view
                    .handle(Command::Editor(EditorCommands::SecondElapsed))?;
            }
            io::stdout().flush()?;
        }
        Ok(())
    }

    fn has_second_passed(&self, start_time: DateTime<Utc>) -> bool {
        let now = Utc::now();
        let duration_since = now.signed_duration_since(start_time);
        duration_since >= Duration::seconds(1)
    }
}
