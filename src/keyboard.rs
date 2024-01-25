use crossterm::event;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::commands::{Command, EditorCommands};
use crate::editor::EditorModes;
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
            // TODO: we should have user defined keybindings
            event::Event::Key(event::KeyEvent { code, .. }) => match code {
                event::KeyCode::Esc => {
                    self.state.borrow().active_pane.borrow_mut().mode = EditorModes::Normal;
                }
                event::KeyCode::Char(char) => match char {
                    c if c == 'q'
                        && self.state.borrow().active_pane.borrow().mode == EditorModes::Normal =>
                    {
                        self.commands.get(&EditorCommands::Quit).unwrap().execute();
                    }
                    c if c == 'h'
                        && self.state.borrow().active_pane.borrow().mode == EditorModes::Normal =>
                    {
                        self.commands
                            .get(&EditorCommands::MoveLeft)
                            .unwrap()
                            .execute();
                    }
                    c if c == 'j'
                        && self.state.borrow().active_pane.borrow().mode == EditorModes::Normal =>
                    {
                        self.commands
                            .get(&EditorCommands::MoveDown)
                            .unwrap()
                            .execute();
                    }
                    c if c == 'k'
                        && self.state.borrow().active_pane.borrow().mode == EditorModes::Normal =>
                    {
                        self.commands
                            .get(&EditorCommands::MoveUp)
                            .unwrap()
                            .execute();
                    }
                    c if c == 'l'
                        && self.state.borrow().active_pane.borrow().mode == EditorModes::Normal =>
                    {
                        self.commands
                            .get(&EditorCommands::MoveRight)
                            .unwrap()
                            .execute();
                    }
                    c if c == 'i'
                        && self.state.borrow().active_pane.borrow().mode == EditorModes::Normal =>
                    {
                        self.state.borrow().active_pane.borrow_mut().mode = EditorModes::Insert;
                    }
                    c if c == 'o'
                        && self.state.borrow().active_pane.borrow().mode == EditorModes::Normal =>
                    {
                        self.commands
                            .get(&EditorCommands::InsertLineBelow)
                            .unwrap()
                            .execute();
                    }
                    c if c == 'O'
                        && self.state.borrow().active_pane.borrow().mode == EditorModes::Normal =>
                    {
                        self.commands
                            .get(&EditorCommands::InsertLineAbove)
                            .unwrap()
                            .execute();
                    }
                    c if self.state.borrow().active_pane.borrow().mode == EditorModes::Insert => {
                        self.state.borrow().active_pane.borrow_mut().insert_char(c);
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }
        Ok(())
    }
}
