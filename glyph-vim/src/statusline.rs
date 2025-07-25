use glyph_core::status_provider::{StatuslineContext, StatuslineProvider};

#[derive(Debug)]
pub struct VimStatusline;

impl StatuslineProvider for VimStatusline {
    fn render_statusline(&self, ctx: &StatuslineContext<'_>) -> String {
        let mode = ctx.current_mode.expect_vim();
        let file_name = ctx
            .buffer_info
            .path()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();

        let mode_str = format!("[ {mode} ]");
        let cursor_pos_str = format!(
            "{}:{}",
            ctx.cursor_position.y + 1,
            ctx.cursor_position.x + 1
        );

        let left_side = format!(" {mode_str} {file_name}");
        let right_side = format!("{cursor_pos_str} ");
        let padding = ctx.width - left_side.len() - right_side.len();
        let padding = " ".repeat(padding).to_string();

        format!("{left_side}{padding}{right_side}")
    }
}