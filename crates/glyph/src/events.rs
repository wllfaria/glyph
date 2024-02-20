use crossterm::event::Event;
use crossterm::event::{Event::Key, KeyCode, KeyEvent, KeyModifiers};

use crate::command::{BufferCommands, Command, CursorCommands, EditorCommands};

#[derive(Default)]
pub struct Events {}

impl Events {
    pub fn new() -> Self {
        Events {}
    }

    pub fn handle(&self, event: Event) -> Option<Command> {
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            let command = match code {
                KeyCode::Enter => Some(Command::Buffer(BufferCommands::NewLine)),
                KeyCode::Backspace => Some(Command::Buffer(BufferCommands::Backspace)),
                KeyCode::Left => Some(Command::Cursor(CursorCommands::MoveLeft)),
                KeyCode::Down => Some(Command::Cursor(CursorCommands::MoveDown)),
                KeyCode::Up => Some(Command::Cursor(CursorCommands::MoveUp)),
                KeyCode::Right => Some(Command::Cursor(CursorCommands::MoveRight)),
                KeyCode::Char('K') if modifiers == KeyModifiers::SHIFT => {
                    Some(Command::Buffer(BufferCommands::Hover))
                }
                KeyCode::Char('s') if modifiers == KeyModifiers::CONTROL => {
                    Some(Command::Buffer(BufferCommands::Save))
                }
                KeyCode::Char('q') if modifiers == KeyModifiers::CONTROL => {
                    Some(Command::Editor(EditorCommands::Quit))
                }
                KeyCode::Char(c) => Some(Command::Buffer(BufferCommands::Type(c))),
                _ => None,
            };
            return command;
        }
        None
    }
}
