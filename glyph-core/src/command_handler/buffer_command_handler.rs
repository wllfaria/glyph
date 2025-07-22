use super::{CommandContext, CommandHandler, CommandHandlerResult};
use crate::key_mapper::Command;

#[derive(Debug, Default)]
pub struct BufferCommandHandler {}

impl CommandHandler for BufferCommandHandler {
    fn handle_commands(&mut self, ctx: &mut CommandContext<'_>) -> CommandHandlerResult {
        let view = ctx.views.get_mut_active_view();
        let cursor = view.cursors.first_mut().unwrap();

        for command in ctx.commands.iter() {
            match command {
                Command::MoveCursorLeft => cursor.x = cursor.x.saturating_sub(1),
                Command::MoveCursorDown => cursor.y = cursor.y.saturating_add(1),
                Command::MoveCursorUp => cursor.y = cursor.y.saturating_sub(1),
                Command::MoveCursorRight => cursor.x = cursor.x.saturating_add(1),
                // TODO: this should be temporary
                Command::Quit => *ctx.should_quit = true,
            }
        }

        CommandHandlerResult::Consumed
    }
}
