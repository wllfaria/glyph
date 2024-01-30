use crossterm::event::{self, KeyModifiers};
use std::collections::HashMap;

use crate::commands::{Command, EditorCommands};

pub struct Keyboard {
    pub commands: HashMap<EditorCommands, Box<dyn Command>>,
}

impl Keyboard {
    pub fn new(commands: HashMap<EditorCommands, Box<dyn Command>>) -> Self {
        Keyboard { commands }
    }

    pub fn poll_events(&mut self) -> std::io::Result<()> {
        let event = event::read()?;
        if let event::Event::Key(event::KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                c if c == event::KeyCode::Char('q') && modifiers == KeyModifiers::CONTROL => {
                    self.commands
                        .get(&EditorCommands::Quit)
                        .unwrap()
                        .execute(None);
                }
                event::KeyCode::Enter => {
                    self.commands
                        .get(&EditorCommands::InsertLineBelow)
                        .unwrap()
                        .execute(None);
                }
                event::KeyCode::Backspace => {
                    self.commands
                        .get(&EditorCommands::Backspace)
                        .unwrap()
                        .execute(None);
                }
                event::KeyCode::Left => {
                    self.commands
                        .get(&EditorCommands::MoveLeft)
                        .unwrap()
                        .execute(None);
                }
                event::KeyCode::Down => {
                    self.commands
                        .get(&EditorCommands::MoveDown)
                        .unwrap()
                        .execute(None);
                }
                event::KeyCode::Up => {
                    self.commands
                        .get(&EditorCommands::MoveUp)
                        .unwrap()
                        .execute(None);
                }
                event::KeyCode::Right => {
                    self.commands
                        .get(&EditorCommands::MoveRight)
                        .unwrap()
                        .execute(None);
                }
                event::KeyCode::Char(c) => {
                    self.commands
                        .get(&EditorCommands::Type)
                        .unwrap()
                        .execute(Some(Box::new(c)));
                }
                _ => (),
            }
        }
        Ok(())
    }
}
