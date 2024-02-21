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
                    _ => {}
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

    pub fn handle_normal_event(&self, event: &Event) -> Option<KeyAction> {
        if let Some(action) = self.map_event_to_key_action(&self.config.keys.normal, event) {
            match action {
                KeyAction::Single(_) => return Some(action),
                KeyAction::Multiple(_) => return Some(action),
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
    ) -> Option<KeyAction> {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => {
                let key = match code {
                    KeyCode::Char(c) => format!("{c}"),
                    _ => format!("{code:?}"),
                };

                logger::debug!("{key}");
                let key = match *modifiers {
                    KeyModifiers::ALT => format!("A-{key}"),
                    KeyModifiers::CONTROL => format!("C-{key}"),
                    KeyModifiers::SHIFT => format!("S-{key}"),
                    _ => key,
                };

                mappings.get(&key).cloned()
            }
            _ => None,
        }
    }
}
