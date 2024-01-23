use crossterm::event;

pub struct EventHandler {
    pub is_quitting: bool,
}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler { is_quitting: false }
    }

    pub fn poll_events(&mut self) -> std::io::Result<()> {
        let event = event::read()?;
        match event {
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('q'),
                ..
            }) => self.is_quitting = true,
            _ => (),
        };
        Ok(())
    }
}
