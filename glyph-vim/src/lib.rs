mod key_mapper;

use glyph_core::buffer_manager::Buffer;
use glyph_core::command_handler::{CommandContext, CommandHandler, CommandHandlerResult};
use glyph_core::cursor::Cursor;
use glyph_core::key_mapper::{Command, VimMode};

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
                Command::MoveCursorLineStart => cursor.move_to_line_start(),
                Command::MoveCursorLineEnd => move_cursor_to_line_end(cursor, buffer, mode),
                Command::DeleteWholeLine => delete_whole_line(cursor, buffer),

                // TODO: this should be temporary
                Command::Quit => *ctx.should_quit = true,
            }
        }

        CommandHandlerResult::Consumed
    }
}

fn move_cursor_left(cursor: &mut Cursor) {
    cursor.move_left_by(1);
}

fn move_cursor_down(cursor: &mut Cursor, buffer: &Buffer, mode: VimMode) {
    cursor.move_down_by(buffer, 1);
    adjust_cursor_after_vertical_move(cursor, buffer, mode);
}

fn move_cursor_up(cursor: &mut Cursor, buffer: &Buffer, mode: VimMode) {
    cursor.move_up_by(1);
    adjust_cursor_after_vertical_move(cursor, buffer, mode);
}

fn move_cursor_right(cursor: &mut Cursor, buffer: &Buffer, mode: VimMode) {
    let last_char = buffer
        .content()
        .line(cursor.y)
        .chars()
        .last()
        .unwrap_or_default();
    let has_newline = matches!(last_char, '\n');
    let offset_from_eol = match mode {
        VimMode::Normal if has_newline => 2,
        VimMode::Normal => 1,
        VimMode::Insert => 0,
        VimMode::Visual => 0,
    };

    cursor.move_right_by_with_offset(buffer, 1, offset_from_eol);
}

fn move_cursor_to_line_end(cursor: &mut Cursor, buffer: &Buffer, mode: VimMode) {
    let last_char = buffer
        .content()
        .line(cursor.y)
        .chars()
        .last()
        .unwrap_or_default();
    let has_newline = matches!(last_char, '\n');
    let offset_from_eol = match mode {
        VimMode::Normal if has_newline => 2,
        VimMode::Normal => 1,
        VimMode::Insert => 0,
        VimMode::Visual => 0,
    };

    cursor.move_to_line_end_with_offset(buffer, offset_from_eol);
}

fn delete_whole_line(cursor: &mut Cursor, buffer: &mut Buffer) {
    let content = buffer.content_mut();
    content.delete_whole_line(cursor.y);
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