use crossterm::event::{poll, read, Event::Key, KeyCode, KeyEvent, KeyModifiers};
use std::io;
use std::time;

use crate::command::{BufferCommands, Command, CursorCommands, EditorCommands};

#[derive(Default)]
pub struct Events {}

impl Events {
    pub fn new() -> Self {
        Events {}
    }

    pub fn poll(&self) -> io::Result<Option<Command>> {
        if poll(time::Duration::from_millis(0))? {
            if let Key(KeyEvent {
                code, modifiers, ..
            }) = read()?
            {
                let command = match code {
                    KeyCode::Enter => Some(Command::Buffer(BufferCommands::NewLine)),
                    KeyCode::Backspace => Some(Command::Buffer(BufferCommands::Backspace)),
                    KeyCode::Left => Some(Command::Cursor(CursorCommands::MoveLeft)),
                    KeyCode::Down => Some(Command::Cursor(CursorCommands::MoveDown)),
                    KeyCode::Up => Some(Command::Cursor(CursorCommands::MoveUp)),
                    KeyCode::Right => Some(Command::Cursor(CursorCommands::MoveRight)),
                    c if c == KeyCode::Char('s') && modifiers == KeyModifiers::CONTROL => {
                        Some(Command::Buffer(BufferCommands::Save))
                    }
                    c if c == KeyCode::Char('q') && modifiers == KeyModifiers::CONTROL => {
                        Some(Command::Editor(EditorCommands::Quit))
                    }
                    KeyCode::Char(c) => Some(Command::Buffer(BufferCommands::Type(c))),
                    _ => None,
                };
                return Ok(command);
            }
        }
        Ok(None)
    }
}
