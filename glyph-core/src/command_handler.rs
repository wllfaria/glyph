use std::collections::{BTreeMap, VecDeque};
use std::fmt::Debug;

use crate::buffer_manager::{Buffer, BufferId};
use crate::key_mapper::ResolvedKeymap;
use crate::view_manager::ViewManager;

pub enum CommandHandlerResult {
    Consumed,
    NotConsumed,
}

pub struct CommandContext<'ctx> {
    pub resolved_keymap: &'ctx ResolvedKeymap,
    pub buffers: &'ctx mut BTreeMap<BufferId, Buffer>,
    pub views: &'ctx mut ViewManager,
    pub should_quit: &'ctx mut bool,
}

pub trait CommandHandler: Debug {
    fn handle_commands(&mut self, ctx: &mut CommandContext<'_>) -> CommandHandlerResult;
}

#[derive(Debug, Default)]
pub struct CommandHandlerChain {
    handlers: VecDeque<Box<dyn CommandHandler>>,
}

impl CommandHandlerChain {
    pub fn new() -> Self {
        Self {
            handlers: VecDeque::new(),
        }
    }

    pub fn add_handler(&mut self, handler: Box<dyn CommandHandler>) {
        self.handlers.push_front(handler);
    }
}

impl CommandHandler for CommandHandlerChain {
    fn handle_commands(&mut self, ctx: &mut CommandContext<'_>) -> CommandHandlerResult {
        for handler in self.handlers.iter_mut() {
            if let CommandHandlerResult::Consumed = handler.handle_commands(ctx) {
                return CommandHandlerResult::Consumed;
            }
        }

        CommandHandlerResult::NotConsumed
    }
}
