use std::{cell::RefCell, rc::Rc};

use crate::{commands::Command, state::State};

use super::Directions;

pub struct InsertLineCommand {
    state: Rc<RefCell<State>>,
    direction: Directions,
}

impl Command for InsertLineCommand {
    fn execute(&self) {
        let mut state = self.state.borrow_mut();
        let pane = state.active_pane.clone();
        let pane = match pane {
            Some(ref pane) => pane.borrow(),
            None => return,
        };
        let line = match self.direction {
            Directions::Up => pane.cursor.y as usize,
            _ => pane.cursor.y as usize + 1,
        };
        match state.active_buffer {
            Some(ref buffer) => buffer.lock().unwrap().new_line(line),
            // TODO: this should quit with an error if it happens
            //       but it should never happen.
            None => state.is_quitting = true,
        }
    }
}

impl InsertLineCommand {
    pub fn new(state: Rc<RefCell<State>>, direction: Directions) -> Self {
        Self { state, direction }
    }
}
