mod key_mapper;

use glyph_core::buffer_manager::Buffer;
use glyph_core::command_handler::{CommandContext, CommandHandler, CommandHandlerResult};
use glyph_core::key_mapper::{Command, VimMode};
use glyph_core::view_manager::Cursor;

pub use crate::key_mapper::VimKeymapper;

#[derive(Debug)]
pub struct VimBufferCommandHandler;

impl CommandHandler for VimBufferCommandHandler {
    fn handle_commands(&mut self, ctx: &mut CommandContext<'_>) -> CommandHandlerResult {
        let view = ctx.views.get_mut_active_view();
        let cursor = view.cursors.first_mut().unwrap();
        let buffer = ctx
            .buffers
            .get_mut(&view.buffer_id)
            .expect("view references non-existing buffer");

        let mode = ctx
            .resolved_keymap
            .mode
            .expect("editor mode must be defined in vim mode")
            .expect_vim();

        for command in ctx.resolved_keymap.commands.iter() {
            match command {
                Command::MoveCursorLeft => move_cursor_left(cursor),
                Command::MoveCursorDown => move_cursor_down(cursor, buffer, mode),
                Command::MoveCursorUp => move_cursor_up(cursor, buffer, mode),
                Command::MoveCursorRight => move_cursor_right(cursor, buffer, mode),

                // TODO: this should be temporary
                Command::Quit => *ctx.should_quit = true,
            }
        }

        CommandHandlerResult::Consumed
    }
}

fn move_cursor_left(cursor: &mut Cursor) {
    cursor.x = cursor.x.saturating_sub(1);
    cursor.virtual_x = cursor.x;
}

fn move_cursor_down(cursor: &mut Cursor, buffer: &Buffer, mode: VimMode) {
    let content = buffer.content();
    let total_lines = content.len_lines();
    cursor.y = usize::min(cursor.y + 1, total_lines.saturating_sub(1));
    adjust_cursor_after_vertical_move(cursor, buffer, mode);
}

fn move_cursor_up(cursor: &mut Cursor, buffer: &Buffer, mode: VimMode) {
    cursor.y = cursor.y.saturating_sub(1);
    adjust_cursor_after_vertical_move(cursor, buffer, mode);
}

fn move_cursor_right(cursor: &mut Cursor, buffer: &Buffer, mode: VimMode) {
    let content = buffer.content();
    let line_len = content.line_len(cursor.y);
    let last_char = content.line(cursor.y).chars().last().unwrap_or_default();
    let has_newline = matches!(last_char, '\n');

    let offset_from_eol = match mode {
        VimMode::Normal if has_newline => 2,
        VimMode::Normal => 1,
        VimMode::Insert => 0,
        VimMode::Visual => 0,
    };

    let max_x = line_len.saturating_sub(offset_from_eol);
    cursor.x = usize::min(cursor.x.saturating_add(1), max_x);
    cursor.virtual_x = cursor.x;
}

fn adjust_cursor_after_vertical_move(cursor: &mut Cursor, buffer: &Buffer, mode: VimMode) {
    let content = buffer.content();
    let line_len = content.line_len(cursor.y);
    let last_char = content.line(cursor.y).chars().last().unwrap_or_default();
    let has_newline = matches!(last_char, '\n');

    let offset_from_eol = match mode {
        VimMode::Normal if has_newline => 2,
        VimMode::Normal => 1,
        VimMode::Insert => 0,
        VimMode::Visual => 0,
    };

    let max_x = line_len.saturating_sub(offset_from_eol);

    if cursor.x >= line_len && cursor.x >= cursor.virtual_x {
        cursor.virtual_x = cursor.x;
        cursor.x = max_x;
        return;
    }

    if cursor.x < cursor.virtual_x {
        cursor.x = usize::min(cursor.virtual_x, max_x);
    }
}