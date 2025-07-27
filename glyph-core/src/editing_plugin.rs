use std::fmt::Debug;

use crate::command_handler::CommandHandler;
use crate::key_mapper::Keymapper;
use crate::status_provider::StatuslineProvider;

pub trait EditingPlugin: StatuslineProvider + Keymapper + Debug {
    fn create_command_handler(&self) -> Box<dyn CommandHandler>;
}