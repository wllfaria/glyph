use std::{cell::RefCell, rc::Rc};

use crate::{commands::Command, state::State};

pub enum Directions {
    Up,
    Down,
    Left,
    Right,
}

pub struct MoveCommand {
    pub state: Rc<RefCell<State>>,
    pub direction: Directions,
}

impl Command for MoveCommand {
    fn execute(&self) {
        self.state
            .borrow_mut()
            .active_pane
            .borrow_mut()
            .move_cursor(&self.direction);
    }
}

impl MoveCommand {
    pub fn new(state: Rc<RefCell<State>>, direction: Directions) -> Self {
        Self { state, direction }
    }
}
