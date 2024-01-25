use crossterm::event;
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
        match event {
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('q'),
                ..
            }) => self.commands.get(&EditorCommands::Quit).unwrap().execute(),
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('h'),
                ..
            }) => self
                .commands
                .get(&EditorCommands::MoveLeft)
                .unwrap()
                .execute(),
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('j'),
                ..
            }) => self
                .commands
                .get(&EditorCommands::MoveDown)
                .unwrap()
                .execute(),
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('k'),
                ..
            }) => self
                .commands
                .get(&EditorCommands::MoveUp)
                .unwrap()
                .execute(),
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('l'),
                ..
            }) => self
                .commands
                .get(&EditorCommands::MoveRight)
                .unwrap()
                .execute(),
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('o'),
                ..
            }) => self
                .commands
                .get(&EditorCommands::InsertLineBelow)
                .unwrap()
                .execute(),
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('O'),
                modifiers: event::KeyModifiers::SHIFT,
                ..
            }) => self
                .commands
                .get(&EditorCommands::InsertLineAbove)
                .unwrap()
                .execute(),
            _ => {
                // extract the char from the event
                let char = match event {
                    event::Event::Key(event::KeyEvent { code, .. }) => match code {
                        event::KeyCode::Char(c) => c,
                        _ => ' ',
                    },
                    _ => ' ',
                };
                self.state
                    .borrow()
                    .active_pane
                    .borrow_mut()
                    .insert_char(char);
            }
        }
        Ok(())
    }
}
