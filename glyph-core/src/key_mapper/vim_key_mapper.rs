use crate::event_loop::event::Event;
use crate::key_mapper::{Command, KeyMapper};

#[derive(Debug, Default)]
pub struct VimKeyMapper {
    buffered_key: String,
}

impl VimKeyMapper {
    pub fn new() -> Self {
        Self {
            buffered_key: String::new(),
        }
    }
}

impl KeyMapper for VimKeyMapper {
    fn parse_event(&mut self, event: Option<Event>) -> Option<Command> {
        tracing::debug!("event is {event:?}");
        None
    }
}
