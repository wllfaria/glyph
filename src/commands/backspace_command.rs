use std::{cell::RefCell, rc::Rc};

use crate::{commands::Command, pane::Position, state::State};

pub struct BackspaceCommand {
    state: Rc<RefCell<State>>,
}

impl Command for BackspaceCommand {
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

        let Position {
            render_col, row, ..
        } = &active_pane.cursor;
        let updated_cursor = active_buffer.delete_char(*row as usize, *render_col as usize);
        std::mem::drop(active_buffer);

        active_pane.set_cursor(updated_cursor);
    }
}

impl BackspaceCommand {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self { state }
    }
}
