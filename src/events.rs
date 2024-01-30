use std::{io::Result, time::Duration};

use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::command::{BufferCommands, Command, CursorCommands, EditorCommands};

#[derive(Default)]
pub struct Events {}

impl Events {
    pub fn new() -> Self {
        Events {}
    }

    pub fn poll_events(&self) -> Result<Option<Command>> {
        if poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = read()?
            {
                let command = match code {
                    KeyCode::Enter => Some(Command::Buffer(BufferCommands::NewLineBelow)),
                    KeyCode::Backspace => Some(Command::Buffer(BufferCommands::Backspace)),
                    KeyCode::Left => Some(Command::Cursor(CursorCommands::MoveLeft)),
                    KeyCode::Down => Some(Command::Cursor(CursorCommands::MoveDown)),
                    KeyCode::Up => Some(Command::Cursor(CursorCommands::MoveUp)),
                    KeyCode::Right => Some(Command::Cursor(CursorCommands::MoveRight)),
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
