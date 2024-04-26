use std::io::stdout;

use crate::{
    config::{Action, KeyAction},
    cursor::Cursor,
    editor::Mode,
    frame::Frame,
    theme::Theme,
};

use super::{rect::Rect, Renderable};

pub struct Commandline<'a> {
    size: Rect,
    theme: &'a Theme,
    cursor: Cursor,
    command_kind: CommandKind,
    command: String,
}

pub enum CommandKind {
    None,
    Command,
    Search,
}

impl std::fmt::Display for CommandKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandKind::None => write!(f, ""),
            CommandKind::Command => write!(f, ":"),
            CommandKind::Search => write!(f, "/"),
        }
    }
}

impl<'a> Commandline<'a> {
    pub fn new(size: Rect, theme: &'a Theme) -> Self {
        let cursor = Cursor {
            row: size.y as usize,
            col: 1,
            absolute_position: 0,
        };

        Self {
            size,
            theme,
            cursor,
            command_kind: CommandKind::None,
            command: String::default(),
        }
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn render_cursor(&self) -> anyhow::Result<()> {
        crossterm::queue!(
            stdout(),
            crossterm::cursor::MoveTo(self.cursor.col as u16, self.cursor.row as u16)
        )?;

        Ok(())
    }

    pub fn update_kind(&mut self, kind: CommandKind) {
        self.command_kind = kind;
    }

    pub fn handle_action(&mut self, action: &KeyAction) -> Option<Action> {
        match action {
            KeyAction::Simple(Action::InsertCommand(c)) => {
                self.cursor.col += 1;
                self.command.push(*c);
            }
            KeyAction::Simple(Action::DeletePreviousChar) => {
                if self.command.is_empty() {
                    self.command_kind = CommandKind::None;
                    return Some(Action::EnterMode(Mode::Normal));
                }
                self.cursor.col = self.cursor.col.saturating_sub(1);
                self.command.pop();
            }
            _ => {}
        };

        None
    }
}

impl Renderable<'_> for Commandline<'_> {
    fn render(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        let command = format!(
            "{}{}{}",
            self.command_kind,
            self.command,
            " ".repeat(
                self.size
                    .width
                    .saturating_sub(self.command.len() as u16 + 1) as usize
            )
        );

        frame.set_text(0, self.size.y, &command, &self.theme.appearance);

        Ok(())
    }

    fn resize(&mut self, new_area: Rect) -> anyhow::Result<()> {
        Ok(())
    }
}
