use std::{cell::RefCell, rc::Rc};

use crate::{commands::Command, state::State};

use super::Directions;

pub struct InsertLineCommand {
    state: Rc<RefCell<State>>,
    direction: Directions,
}

impl Command for InsertLineCommand {
    fn execute(&self) {
        self.state
            .borrow_mut()
            .active_pane
            .borrow_mut()
            .insert_line(&self.direction);
    }
}

impl InsertLineCommand {
    pub fn new(state: Rc<RefCell<State>>, direction: Directions) -> Self {
        Self { state, direction }
    }
}
