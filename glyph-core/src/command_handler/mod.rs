pub mod buffer_command_handler;

use std::collections::{BTreeMap, VecDeque};
use std::fmt::Debug;

use crate::buffer_manager::{Buffer, BufferId};
use crate::key_mapper::Command;
use crate::view_manager::{View, ViewManager};

pub enum CommandHandlerResult {
    Consumed,
    NotConsumed,
}

pub struct CommandContext<'ctx> {
    pub commands: &'ctx [Command],
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

    pub fn add_handler<H>(&mut self, handler: H)
    where
        H: CommandHandler + 'static,
    {
        self.handlers.push_front(Box::new(handler));
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
