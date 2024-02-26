use std::collections::HashMap;

use crossterm::event::Event;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::{Action, Config, KeyAction};
use crate::editor::Mode;

#[derive(Debug)]
pub struct Events<'a> {
    action_being_composed: Option<String>,
    config: &'a Config,
}

impl<'a> Events<'a> {
    pub fn new(config: &'a Config) -> Self {
        Events {
            action_being_composed: None,
            config,
        }
    }

    pub fn handle(&mut self, event: &Event, mode: &Mode) -> Option<KeyAction> {
        if let Some(action) = self.action_being_composed.take() {
            self.action_being_composed = Some(action);
            match event {
                Event::Key(KeyEvent { code, .. }) => {
                    let action = self.action_being_composed.clone().unwrap();
                    let action = self.config.keys.normal.get(&action).unwrap();
                    let key = match code {
                        KeyCode::Char(c) => *c,
                        _ => ' ',
                    };

                    match action {
                        KeyAction::Complex(complex) => {
                            let action = complex.get(key.to_string().as_str());
                            tracing::trace!("{action:?}");
                            if let Some(action) = action {
                                self.action_being_composed = None;
                                return Some(action.clone());
                            }
                            self.action_being_composed = None;
                        }
                        _ => {
                            self.action_being_composed = None;
                        }
                    }
                }
                _ => {
                    self.action_being_composed = None;
                }
            }
        }

        match mode {
            Mode::Normal => self.handle_normal_event(event),
            Mode::Insert => self.handle_insert_event(event),
            Mode::Command => self.handle_command_event(event),
            Mode::Search => self.handle_search_event(event),
        }
    }

    pub fn handle_normal_event(&mut self, event: &Event) -> Option<KeyAction> {
        let (key, action) = self.map_event_to_key_action(&self.config.keys.normal, event);
        tracing::trace!("{action:?}");
        if let Some(action) = action {
            match action {
                KeyAction::Simple(_) => return Some(action),
                KeyAction::Multiple(_) => return Some(action),
                KeyAction::Complex(_) => {
                    self.action_being_composed = key;
                    return None;
                }
            };
        };
        None
    }
    pub fn handle_insert_event(&self, event: &Event) -> Option<KeyAction> {
        let (_, action) = self.map_event_to_key_action(&self.config.keys.insert, event);
        if let Some(action) = action {
            match action {
                KeyAction::Simple(_) => return Some(action),
                KeyAction::Multiple(_) => return Some(action),
                KeyAction::Complex(_) => return Some(action),
            }
        };

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) => Some(KeyAction::Simple(Action::InsertChar(*c))),
            _ => None,
        }
    }
    pub fn handle_command_event(&self, event: &Event) -> Option<KeyAction> {
        tracing::debug!("handling command event");
        let (_, action) = self.map_event_to_key_action(&self.config.keys.command, event);
        if let Some(action) = action {
            match action {
                KeyAction::Simple(_) => return Some(action),
                KeyAction::Multiple(_) => return Some(action),
                KeyAction::Complex(_) => return Some(action),
            }
        };
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) => Some(KeyAction::Simple(Action::InsertCommand(*c))),
            _ => None,
        }
    }
    pub fn handle_search_event(&self, _event: &Event) -> Option<KeyAction> {
        None
    }

    pub fn map_event_to_key_action(
        &self,
        mappings: &HashMap<String, KeyAction>,
        event: &Event,
    ) -> (Option<String>, Option<KeyAction>) {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => {
                let key = match code {
                    KeyCode::Char(c) => format!("{c}"),
                    _ => format!("{code:?}"),
                };

                let key = match *modifiers {
                    KeyModifiers::ALT => format!("A-{key}"),
                    KeyModifiers::CONTROL => format!("C-{key}"),
                    KeyModifiers::SHIFT => format!("S-{key}"),
                    _ => key,
                };

                (Some(key.clone()), mappings.get(&key).cloned())
            }
            _ => (None, None),
        }
    }
}
