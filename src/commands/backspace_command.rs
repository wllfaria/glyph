use std::{cell::RefCell, rc::Rc};

use crate::{commands::Command, state::State};

pub struct BackspaceCommand {
    state: Rc<RefCell<State>>,
}

impl Command for BackspaceCommand {
    fn execute(&self) {
        let mut state = self.state.borrow_mut();
        let pane = state.active_pane.clone();
        let pane = match pane {
            Some(ref pane) => pane.borrow(),
            None => return,
        };
        match state.active_buffer {
            Some(ref buffer) => buffer
                .lock()
                .unwrap()
                .delete_char(pane.cursor.y as usize, pane.cursor.x as usize - 1),
            // TODO: this should quit with an error if it happens
            //       but it should never happen.
            None => state.is_quitting = true,
        }
    }
}

impl BackspaceCommand {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self { state }
    }
}
