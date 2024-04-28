use config::{Action, KeyAction, Mode};
use theme::Theme;

use crate::{
    cursor::Cursor,
    frame::Frame,
    tui::{rect::Rect, Renderable},
};

use std::{io::stdout, ops::Add};

pub struct Commandline<'a> {
    size: Rect,
    theme: &'a Theme,
    cursor: Cursor,
    command_kind: CommandKind,
    command: String,
    message: String,
}

pub enum CommandKind {
    None,
    Command,
    Search,
    Message,
}

impl std::fmt::Display for CommandKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandKind::None => write!(f, ""),
            CommandKind::Command => write!(f, ":"),
            CommandKind::Search => write!(f, "/"),
            CommandKind::Message => write!(f, ""),
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
            message: String::default(),
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

    pub fn clear(&mut self) {
        self.command.clear();
        self.cursor.absolute_position = 0;
        self.cursor.col = self.size.x.add(1).into();
        self.cursor.row = self.size.y.into();
        self.command_kind = CommandKind::None;
        self.clear_message();
    }

    pub fn clear_message(&mut self) {
        self.message.clear();
    }

    pub fn update_kind(&mut self, kind: CommandKind) {
        self.command_kind = kind;
    }

    pub fn handle_action(&mut self, action: &KeyAction) -> Option<Action> {
        match action {
            KeyAction::Simple(Action::ShowMessage(message)) => {
                self.command_kind = CommandKind::Message;
                self.message = message.to_string();
            }
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

    fn build_content(&self) -> String {
        match self.command_kind {
            CommandKind::Command => format!(
                "{}{}{}",
                self.command_kind,
                self.command,
                " ".repeat(usize::from(self.size.width).saturating_sub(self.command.len().add(1)))
            ),
            CommandKind::Message => format!(
                "{}{}",
                self.message,
                " ".repeat(usize::from(self.size.width).saturating_sub(self.message.len()))
            ),
            _ => " ".repeat(usize::from(self.size.width)),
        }
    }
}

impl Renderable<'_> for Commandline<'_> {
    fn render(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        let content = self.build_content();
        frame.set_text(0, self.size.y, &content, &self.theme.appearance);

        Ok(())
    }

    fn resize(&mut self, new_size: Rect) -> anyhow::Result<()> {
        self.size = new_size;
        Ok(())
    }
}
