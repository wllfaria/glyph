use crossterm::event;
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
        if event::poll(time::Duration::from_millis(0))? {
            if let event::Event::Key(event::KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                let command = match code {
                    event::KeyCode::Enter => Some(Command::Buffer(BufferCommands::NewLineBelow)),
                    event::KeyCode::Backspace => Some(Command::Buffer(BufferCommands::Backspace)),
                    event::KeyCode::Left => Some(Command::Cursor(CursorCommands::MoveLeft)),
                    event::KeyCode::Down => Some(Command::Cursor(CursorCommands::MoveDown)),
                    event::KeyCode::Up => Some(Command::Cursor(CursorCommands::MoveUp)),
                    event::KeyCode::Right => Some(Command::Cursor(CursorCommands::MoveRight)),
                    c if c == event::KeyCode::Char('q')
                        && modifiers == event::KeyModifiers::CONTROL =>
                    {
                        Some(Command::Editor(EditorCommands::Quit))
                    }
                    event::KeyCode::Char(c) => Some(Command::Buffer(BufferCommands::Type(c))),
                    _ => None,
                };
                return Ok(command);
            }
        }
        Ok(None)
    }
}
