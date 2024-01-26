use std::{cell::RefCell, rc::Rc};

use crate::{
    commands::{Command, Directions},
    state::State,
};

pub struct MoveCommand {
    pub state: Rc<RefCell<State>>,
    pub direction: Directions,
}

impl Command for MoveCommand {
    fn execute(&self) {
        let mut state = self.state.borrow_mut();
        match state.active_pane {
            Some(ref pane) => pane.borrow_mut().move_cursor(&self.direction),
            // TODO: this should quit with an error if it happens
            //       but it should never happen.
            None => state.is_quitting = true,
        }
    }
}

impl MoveCommand {
    pub fn new(state: Rc<RefCell<State>>, direction: Directions) -> Self {
        Self { state, direction }
    }
}
