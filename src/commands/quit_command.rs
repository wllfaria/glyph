use std::{cell::RefCell, rc::Rc};

use crate::{commands::Command, state::State};

pub struct QuitCommand {
    pub state: Rc<RefCell<State>>,
}

impl Command for QuitCommand {
    fn execute(&self, _: Option<Box<dyn std::any::Any>>) {
        self.state.borrow_mut().is_quitting = true;
    }
}

impl QuitCommand {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self { state }
    }
}
