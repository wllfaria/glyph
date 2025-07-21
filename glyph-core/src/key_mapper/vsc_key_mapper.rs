use crate::event_loop::event::Event;
use crate::key_mapper::{Command, Keymapper};

#[derive(Debug, Default)]
pub struct VSCodeKeymapper {}

impl VSCodeKeymapper {
    pub fn new() -> Self {
        Self {}
    }
}

impl Keymapper for VSCodeKeymapper {
    fn parse_event(&mut self, _event: Option<Event>) -> Option<Vec<Command>> {
        None
    }
}
