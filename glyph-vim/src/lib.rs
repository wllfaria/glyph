mod key_mapper;
mod statusline;

use glyph_core::buffer_manager::Buffer;
use glyph_core::command_handler::{CommandContext, CommandHandler, CommandHandlerResult};
use glyph_core::cursor::Cursor;
use glyph_core::geometry::Point;
use glyph_core::key_mapper::{Command, VimMode};

pub use crate::key_mapper::VimKeymapper;
pub use crate::statusline::VimStatusline;

#[derive(Debug)]
pub struct VimBufferCommandHandler;

impl CommandHandler for VimBufferCommandHandler {
    fn handle_commands(&mut self, ctx: &mut CommandContext<'_>) -> CommandHandlerResult {
        let mode = ctx
            .resolved_keymap
            .mode
            .expect("editor mode must be defined in vim mode")
            .expect_vim();

        for command in ctx.resolved_keymap.commands.iter() {
            match command {
                Command::MoveCursorLeft => move_cursor_left(ctx),
                Command::MoveCursorDown => move_cursor_down(ctx, mode),
                Command::MoveCursorUp => move_cursor_up(ctx, mode),
                Command::MoveCursorRight => move_cursor_right(ctx, mode),
                Command::MoveCursorRightOverLines => move_cursor_right_over_lines(ctx, mode),
                Command::MoveCursorLineStart => move_cursor_to_line_start(ctx),
                Command::MoveCursorLineEnd => move_cursor_to_line_end(ctx, mode),
                Command::DeleteWholeLine => delete_whole_line(ctx, mode),
                Command::MoveToTop => move_to_top(ctx, mode),
                Command::MoveToBottom => move_to_bottom(ctx, mode),
                Command::PageUp => page_up(ctx, mode),
                Command::PageDown => page_down(ctx, mode),
                Command::MoveToMatchingPair => move_to_matching_pair(ctx, mode),
                Command::MoveToFirstNonSpace => move_to_first_non_space(ctx),
                Command::MoveToLastNonSpace => move_to_last_non_space(ctx),
                Command::MoveToNextParagraph => move_to_next_paragraph(ctx),
                Command::MoveToPrevParagraph => move_to_prev_paragraph(ctx),
                Command::DeletePrevChar => delete_prev_char(ctx, mode),
                Command::DeleteCurrChar => delete_curr_char(ctx, mode),
                Command::TypeChar(c) => insert_character(ctx, mode, *c),
                Command::MoveToNextWord => move_to_next_word(ctx, mode),

                // TODO: this should be temporary
                Command::Quit => *ctx.should_quit = true,
            }
        }

        scroll_view_to_cursor(ctx);

        CommandHandlerResult::Consumed
    }
}

fn move_cursor_left(ctx: &mut CommandContext<'_>) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    cursor.move_left_by(1);
}

fn move_cursor_down(ctx: &mut CommandContext<'_>, mode: VimMode) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    cursor.move_down_by(buffer, 1);
    adjust_cursor_after_vertical_move(cursor, buffer, mode);
}

fn move_cursor_up(ctx: &mut CommandContext<'_>, mode: VimMode) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    cursor.move_up_by(1);
    adjust_cursor_after_vertical_move(cursor, buffer, mode);
}

fn move_cursor_right(ctx: &mut CommandContext<'_>, mode: VimMode) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    let last_char = buffer
        .content()
        .line(cursor.y)
        .chars()
        .last()
        .unwrap_or_default();
    let has_newline = matches!(last_char, '\n');
    let offset_from_eol = get_offset_from_eol(mode, has_newline);

    cursor.move_right_by_with_offset(buffer, 1, offset_from_eol);
}

fn move_cursor_right_over_lines(ctx: &mut CommandContext<'_>, mode: VimMode) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    let last_char = buffer
        .content()
        .line(cursor.y)
        .chars()
        .last()
        .unwrap_or_default();
    let has_newline = matches!(last_char, '\n');
    let offset_from_eol = get_offset_from_eol(mode, has_newline);

    cursor.move_right_by_with_offset(buffer, 1, offset_from_eol);
}

fn get_offset_from_eol(mode: VimMode, has_newline: bool) -> usize {
    match mode {
        VimMode::Normal if has_newline => 2,
        VimMode::Normal => 1,
        VimMode::Insert => 1,
        VimMode::Visual => 0,
        VimMode::Command => 0,
    }
}

fn move_cursor_to_line_start(ctx: &mut CommandContext<'_>) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    cursor.move_to_line_start();
}

fn move_cursor_to_line_end(ctx: &mut CommandContext<'_>, mode: VimMode) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    let last_char = buffer
        .content()
        .line(cursor.y)
        .chars()
        .last()
        .unwrap_or_default();

    let has_newline = matches!(last_char, '\n');
    let offset_from_eol = get_offset_from_eol(mode, has_newline);

    cursor.move_to_line_end_with_offset(buffer, offset_from_eol);
}

fn delete_whole_line(ctx: &mut CommandContext<'_>, mode: VimMode) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    let content = buffer.content_mut();

    let len_lines = content.len_lines();
    let is_last_line = cursor.y == len_lines.saturating_sub(1);
    content.delete_whole_line(cursor.y);

    // When deleting the last line of text, the cursor should move to the line above to not be
    // out-of-bounds from the text buffer
    if is_last_line {
        move_cursor_up(ctx, mode);
    }
}

fn move_to_top(ctx: &mut CommandContext<'_>, mode: VimMode) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    cursor.move_up_by(usize::MAX);
    adjust_cursor_after_vertical_move(cursor, buffer, mode);
}

fn move_to_bottom(ctx: &mut CommandContext<'_>, mode: VimMode) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    cursor.move_down_by(buffer, usize::MAX);
    adjust_cursor_after_vertical_move(cursor, buffer, mode);
}

fn page_up(ctx: &mut CommandContext<'_>, mode: VimMode) {
    let view_id = ctx.views.get_active_view_id();
    let layout = ctx.views.get_layout_for_view(view_id);
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    let half_page = layout.usable_rect.height / 2;
    cursor.move_up_by(half_page as usize);
    adjust_cursor_after_vertical_move(cursor, buffer, mode);
}

fn page_down(ctx: &mut CommandContext<'_>, mode: VimMode) {
    let view_id = ctx.views.get_active_view_id();
    let layout = ctx.views.get_layout_for_view(view_id);
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    let half_page = layout.usable_rect.height / 2;
    cursor.move_down_by(buffer, half_page as usize);
    adjust_cursor_after_vertical_move(cursor, buffer, mode);
}

fn move_to_first_non_space(ctx: &mut CommandContext<'_>) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    let position = buffer.content().find_first_non_space_character(cursor.y);
    cursor.move_to(buffer, position.x, position.y);
}

fn move_to_last_non_space(ctx: &mut CommandContext<'_>) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    let position = buffer.content().find_last_non_space_character(cursor.y);
    cursor.move_to(buffer, position.x, position.y);
}

fn move_to_next_paragraph(ctx: &mut CommandContext<'_>) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    let position = buffer.content().find_next_paragraph(cursor.y);
    cursor.move_to(buffer, position.x, position.y);
}

fn move_to_prev_paragraph(ctx: &mut CommandContext<'_>) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");
    let position = buffer.content().find_prev_paragraph(cursor.y);
    cursor.move_to(buffer, position.x, position.y);
}

fn delete_prev_char(ctx: &mut CommandContext<'_>, mode: VimMode) {
    let view = ctx.views.get_mut_active_view();
    let buffer_id = view.buffer_id;
    let cursor = view.cursors.first_mut().unwrap();
    let position = Point::new(cursor.x, cursor.y);
    let is_cursor_on_line_start = cursor.x == 0;

    match mode {
        VimMode::Insert if !is_cursor_on_line_start => {
            ctx.buffers
                .get_mut(&buffer_id)
                .expect("view references non-existing buffer")
                .content_mut()
                .delete_prev_char(Point::new(cursor.x, cursor.y));
            move_cursor_left(ctx);
        }
        VimMode::Insert => {
            move_cursor_up(ctx, mode);
            move_cursor_to_line_end(ctx, mode);
            move_cursor_left(ctx);
            ctx.buffers
                .get_mut(&buffer_id)
                .expect("view references non-existing buffer")
                .content_mut()
                .delete_prev_char(position);
        }
        VimMode::Normal if !is_cursor_on_line_start => {
            ctx.buffers
                .get_mut(&buffer_id)
                .expect("view references non-existing buffer")
                .content_mut()
                .delete_prev_char(Point::new(cursor.x, cursor.y));
            move_cursor_left(ctx);
        }
        VimMode::Normal => (),
        VimMode::Visual => (),
        VimMode::Command => (),
    }
}

fn delete_curr_char(ctx: &mut CommandContext<'_>, mode: VimMode) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    let content = buffer.content_mut();
    let line_len = content.line_len(cursor.y);
    let last_char = content.line(cursor.y).chars().last().unwrap_or_default();
    let has_newline = matches!(last_char, '\n');
    let is_empty_line = line_len == 1 && has_newline;
    let offset_from_eol = get_offset_from_eol(mode, has_newline);

    match mode {
        VimMode::Normal => {
            if is_empty_line {
                return;
            }

            let is_on_last_char = cursor.x == line_len.saturating_sub(offset_from_eol);
            content.delete_curr_char(Point::new(cursor.x, cursor.y));

            if is_on_last_char {
                move_cursor_left(ctx);
            }
        }
        VimMode::Insert => content.delete_curr_char(Point::new(cursor.x, cursor.y)),
        VimMode::Visual => {}
        VimMode::Command => {}
    }
}

fn insert_character(ctx: &mut CommandContext<'_>, mode: VimMode, ch: char) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let position = Point::new(cursor.x, cursor.y);
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    buffer.content_mut().insert_char_at(position, ch);
    move_cursor_right(ctx, mode);
}

fn move_to_next_word(ctx: &mut CommandContext<'_>, mode: VimMode) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let position = Point::new(cursor.x, cursor.y);
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    let content = buffer.content();
    let position = content.find_next_word_boundary(position);

    let last_char = content.line(position.y).chars().last().unwrap_or_default();
    // TODO: this should also work when EOL is \r or \r\n
    let has_newline = matches!(last_char, '\n');
    let offset_from_eol = get_offset_from_eol(mode, has_newline);

    cursor.move_to_with_offset(buffer, position.x, position.y, offset_from_eol);
}

fn move_to_matching_pair(ctx: &mut CommandContext<'_>, mode: VimMode) {
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();
    let buffer = ctx
        .buffers
        .get_mut(&view.buffer_id)
        .expect("view references non-existing buffer");

    let matching_pair = buffer
        .content()
        .find_matching_pair(Point::new(cursor.x, cursor.y));

    cursor.move_to(buffer, matching_pair.x, matching_pair.y);
    adjust_cursor_after_vertical_move(cursor, buffer, mode);
}

fn adjust_cursor_after_vertical_move(cursor: &mut Cursor, buffer: &Buffer, mode: VimMode) {
    let content = buffer.content();
    let line_len = content.line_len(cursor.y);
    let last_char = content.line(cursor.y).chars().last().unwrap_or_default();
    let has_newline = matches!(last_char, '\n');

    let offset_from_eol = get_offset_from_eol(mode, has_newline);
    let max_x = line_len.saturating_sub(offset_from_eol);
    if cursor.x >= max_x && cursor.x >= cursor.virtual_x {
        cursor.virtual_x = cursor.x;
        cursor.x = max_x;
        return;
    }

    if cursor.x < cursor.virtual_x {
        cursor.x = usize::min(cursor.virtual_x, max_x);
    }
}

fn scroll_view_to_cursor(ctx: &mut CommandContext<'_>) {
    let active_view_id = ctx.views.get_active_view_id();
    let layout = ctx.views.get_layout_for_view(active_view_id);
    let view = ctx.views.get_mut_active_view();
    let cursor = view.cursors.first_mut().unwrap();

    let view_height = layout.usable_rect.height as usize;
    let view_width = layout.usable_rect.width as usize;

    if cursor.y.saturating_sub(view.scroll_offset.y) > view_height - 1 {
        view.scroll_offset.y = cursor.y - (view_height - 1)
    }

    if cursor.y.saturating_sub(view.scroll_offset.y) == 0 {
        let vertical_offset = view.scroll_offset.y.saturating_sub(cursor.y);
        view.scroll_offset.y = view.scroll_offset.y.saturating_sub(vertical_offset);
    }

    if cursor.x.saturating_sub(view.scroll_offset.x) == 0 {
        let horizontal_offset = view.scroll_offset.x.saturating_sub(cursor.x);
        view.scroll_offset.x = view.scroll_offset.x.saturating_sub(horizontal_offset);
    }

    if cursor.x.saturating_sub(view.scroll_offset.x) > view_width - 1 {
        view.scroll_offset.x = cursor.x - (view_width - 1);
    }
}
