use crate::event_loop::event::Event;
use crate::key_mapper::{Command, KeyMapper};

#[derive(Debug, Default)]
pub struct VSCodeKeyMapper {}

impl VSCodeKeyMapper {
    pub fn new() -> Self {
        Self {}
    }
}

impl KeyMapper for VSCodeKeyMapper {
    fn parse_event(&mut self, _event: Option<Event>) -> Option<Command> {
        None
    }
}
