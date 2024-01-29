use std::{cell::RefCell, rc::Rc};

use crate::{commands::Command, state::State};

use super::Directions;

pub struct InsertLineBelowCommand {
    state: Rc<RefCell<State>>,
}

impl Command for InsertLineBelowCommand {
    fn execute(&self, _: Option<Box<dyn std::any::Any>>) {
        let state = self.state.borrow_mut();
        let mut active_buffer = match state.active_buffer {
            Some(ref buffer) => buffer.lock().unwrap(),
            None => panic!("No active buffer!"),
        };
        let mut active_pane = match state.active_pane {
            Some(ref pane) => pane.borrow_mut(),
            None => panic!("No active pane!"),
        };
        let cursor = &active_pane.cursor;
        active_buffer.new_line(cursor.row as usize, cursor.col as usize);
        std::mem::drop(active_buffer);
        active_pane.move_cursor(&Directions::Down);
        active_pane.move_cursor(&Directions::LineStart);
    }
}

impl InsertLineBelowCommand {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self { state }
    }
}
