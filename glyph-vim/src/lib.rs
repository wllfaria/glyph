mod command_handler;
mod key_mapper;
mod statusline;

use glyph_core::command_handler::CommandHandler;
use glyph_core::editing_plugin::EditingPlugin;
use glyph_core::event_loop::Event;
use glyph_core::key_mapper::{EditorMode, Keymapper, ResolvedKeymap, VimMode};
use glyph_core::status_provider::{StatuslineContext, StatuslineProvider};

use crate::command_handler::VimBufferCommandHandler;
use crate::key_mapper::*;
use crate::statusline::VimStatusline;

#[derive(Debug)]
pub struct VimEditingPlugin {
    statusline: VimStatusline,
    normal_mode_keymapper: NormalModeKeymapper,
    insert_mode_keymapper: InsertModeKeymapper,
    mode: VimMode,
}

impl Default for VimEditingPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl VimEditingPlugin {
    pub fn new() -> Self {
        let loaded_keymaps = load_vim_keymaps();

        Self {
            statusline: VimStatusline,
            mode: VimMode::Normal,
            normal_mode_keymapper: NormalModeKeymapper::new(loaded_keymaps.normal),
            insert_mode_keymapper: InsertModeKeymapper::new(loaded_keymaps.insert),
        }
    }
}

impl Keymapper for VimEditingPlugin {
    fn parse_event(&mut self, event: Option<Event>) -> Option<ResolvedKeymap> {
        let Event::Key(key) = event?;

        let commands = match self.mode {
            VimMode::Normal => self.normal_mode_keymapper.handle_key(key),
            VimMode::Insert => self.insert_mode_keymapper.handle_key(key),
            VimMode::Command => todo!(),
            VimMode::Visual => todo!(),
        };

        let mut general_commands = vec![];
        for cmd in commands {
            match cmd {
                CommandWrapper::General(cmd) => general_commands.push(cmd),
                CommandWrapper::Vim(cmd) => match cmd {
                    VimCommand::InsertMode => self.mode = VimMode::Insert,
                    VimCommand::NormalMode => self.mode = VimMode::Normal,
                    VimCommand::CommandMode => self.mode = VimMode::Command,
                },
            }
        }

        Some(ResolvedKeymap {
            commands: general_commands,
            mode: Some(self.mode()),
        })
    }

    fn mode(&self) -> EditorMode {
        EditorMode::Vim(self.mode)
    }
}

impl StatuslineProvider for VimEditingPlugin {
    fn render_statusline(&self, ctx: &StatuslineContext) -> String {
        self.statusline.render_statusline(ctx)
    }
}

impl EditingPlugin for VimEditingPlugin {
    fn create_command_handler(&self) -> Box<dyn CommandHandler> {
        Box::new(VimBufferCommandHandler)
    }
}