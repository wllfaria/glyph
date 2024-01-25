use crossterm::event::{self, KeyModifiers};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::commands::{Command, EditorCommands};
use crate::state::State;

pub struct Keyboard {
    pub state: Rc<RefCell<State>>,
    pub commands: HashMap<EditorCommands, Box<dyn Command>>,
}

impl Keyboard {
    pub fn new(
        state: Rc<RefCell<State>>,
        commands: HashMap<EditorCommands, Box<dyn Command>>,
    ) -> Self {
        Keyboard {
            commands,
            state: Rc::clone(&state),
        }
    }

    pub fn poll_events(&mut self) -> std::io::Result<()> {
        let event = event::read()?;
        if let event::Event::Key(event::KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                c if c == event::KeyCode::Char('q') && modifiers == KeyModifiers::CONTROL => {
                    self.commands.get(&EditorCommands::Quit).unwrap().execute();
                }
                event::KeyCode::Enter => {
                    self.commands
                        .get(&EditorCommands::InsertLineBelow)
                        .unwrap()
                        .execute();
                }
                event::KeyCode::Backspace => {
                    self.commands
                        .get(&EditorCommands::Backspace)
                        .unwrap()
                        .execute();
                }
                event::KeyCode::Left => {
                    self.commands
                        .get(&EditorCommands::MoveLeft)
                        .unwrap()
                        .execute();
                }
                event::KeyCode::Down => {
                    self.commands
                        .get(&EditorCommands::MoveDown)
                        .unwrap()
                        .execute();
                }
                event::KeyCode::Up => {
                    self.commands
                        .get(&EditorCommands::MoveUp)
                        .unwrap()
                        .execute();
                }
                event::KeyCode::Right => {
                    self.commands
                        .get(&EditorCommands::MoveRight)
                        .unwrap()
                        .execute();
                }
                event::KeyCode::Char(char) => self
                    .state
                    .borrow()
                    .active_pane
                    .borrow_mut()
                    .insert_char(char),
                _ => (),
            }
        }
        Ok(())
    }
}
