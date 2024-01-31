use std::io::{stdout, Result, Write};
use std::{thread, time};

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
        let target_fps = 60;
        let target_frame_time = time::Duration::from_millis(1000 / target_fps);

        while self.is_running {
            let frame_start = time::Instant::now();

            match self.events.poll()? {
                Some(Command::Editor(EditorCommands::Quit)) => self.is_running = false,
                Some(command) => self.view.handle_command(command),
                _ => (),
            }
            self.view.render()?;
            stdout().flush()?;

            let frame_duration = frame_start.elapsed();
            if frame_duration < target_frame_time {
                thread::sleep(target_frame_time - frame_duration);
            }
        }

        self.view.shutdown()?;
        Ok(())
    }
}
