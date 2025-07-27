use crate::command_handler::{CommandHandler, CommandHandlerResult};
use crate::key_mapper::Command;

#[derive(Debug)]
pub struct FileCommandHandler;

impl CommandHandler for FileCommandHandler {
    fn handle_commands(&mut self, ctx: &mut super::CommandContext<'_>) -> CommandHandlerResult {
        for command in ctx.resolved_keymap.commands.iter() {
            match command {
                Command::Quit => *ctx.should_quit = true,
                Command::Save => todo!(),
                Command::SaveAll => todo!(),
                _ => {}
            }
        }

        CommandHandlerResult::Consumed
    }
}