use std::collections::HashMap;

use crossterm::event::Event;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::{Config, KeyAction};
use crate::editor::Mode;

pub struct Events<'a> {
    action_being_composed: Option<String>,
    action_modifier: usize,
    config: &'a Config,
}

impl<'a> Events<'a> {
    pub fn new(config: &'a Config) -> Self {
        Events {
            action_being_composed: None,
            action_modifier: 0,
            config,
        }
    }

    pub fn handle(&mut self, event: &Event, mode: &Mode) -> Option<KeyAction> {
        if let Some(action) = self.action_being_composed.take() {
            // we are currently giving the action back to handle modifiers
            // theres probably a better way to do this
            self.action_being_composed = Some(action);
            match event {
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Char('1') => self.action_modifier = self.action_modifier * 10 + 1,
                    KeyCode::Char('2') => self.action_modifier = self.action_modifier * 10 + 2,
                    KeyCode::Char('3') => self.action_modifier = self.action_modifier * 10 + 3,
                    KeyCode::Char('4') => self.action_modifier = self.action_modifier * 10 + 4,
                    KeyCode::Char('5') => self.action_modifier = self.action_modifier * 10 + 5,
                    KeyCode::Char('6') => self.action_modifier = self.action_modifier * 10 + 6,
                    KeyCode::Char('7') => self.action_modifier = self.action_modifier * 10 + 7,
                    KeyCode::Char('8') => self.action_modifier = self.action_modifier * 10 + 8,
                    KeyCode::Char('9') => self.action_modifier = self.action_modifier * 10 + 9,
                    KeyCode::Char('0') => self.action_modifier *= 10,
                    // if it is not a number, we either execute the action
                    // or cancel it.
                    _ => {
                        let action = self.action_being_composed.clone().unwrap();
                        let action = self.config.keys.normal.get(&action).unwrap();
                        let key = match code {
                            KeyCode::Char(c) => *c,
                            _ => ' ',
                        };

                        match action {
                            KeyAction::Nested(nested) => {
                                let action = nested.get(key.to_string().as_str());
                                logger::trace!("{action:?}");
                                if let Some(action) = action {
                                    self.action_being_composed = None;
                                    return Some(action.clone());
                                }
                                self.action_being_composed = None;
                            }
                            _ => (),
                        }
                    }
                },
                _ => {
                    self.action_being_composed = None;
                    self.action_modifier = 0;
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
        if let Some(action) = action {
            logger::trace!("normal event: {action:?}");
            match action {
                KeyAction::Single(_) => return Some(action),
                KeyAction::Multiple(_) => return Some(action),
                KeyAction::Nested(_) => {
                    self.action_being_composed = key;
                    return None;
                }
                _ => return None,
            };
        };
        None
    }
    pub fn handle_insert_event(&self, event: &Event) -> Option<KeyAction> {
        None
    }
    pub fn handle_command_event(&self, event: &Event) -> Option<KeyAction> {
        None
    }
    pub fn handle_search_event(&self, event: &Event) -> Option<KeyAction> {
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
