use std::{cell::RefCell, rc::Rc};

use crate::{commands::Command, state::State};

pub struct BackspaceCommand {
    state: Rc<RefCell<State>>,
}

impl Command for BackspaceCommand {
    fn execute(&self) {
        self.state.borrow().active_pane.borrow_mut().delete_char();
    }
}

impl BackspaceCommand {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self { state }
    }
}
