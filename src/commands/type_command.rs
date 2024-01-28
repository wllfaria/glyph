use std::{cell::RefCell, rc::Rc};

use crate::{commands::Command, state::State};

pub struct TypeCommand {
    pub state: Rc<RefCell<State>>,
}

impl Command for TypeCommand {
    fn execute(&self, payload: Option<Box<dyn std::any::Any>>) {
        let state = self.state.borrow_mut();
        let mut active_buffer = match state.active_buffer {
            Some(ref buffer) => buffer.lock().unwrap(),
            None => panic!("No active buffer!"),
        };
        let mut active_pane = match state.active_pane {
            Some(ref pane) => pane.borrow_mut(),
            None => panic!("No active pane!"),
        };
        let offset = &active_pane.cursor_left_limit;
        let cursor = &active_pane.cursor;
        let char = match payload {
            Some(payload) => payload,
            None => panic!("No payload!"),
        };

        let char = char.downcast::<char>().unwrap();
        active_buffer.insert_char(cursor.y as usize, (cursor.x - offset) as usize, *char);
        std::mem::drop(active_buffer);

        active_pane.move_cursor(&crate::commands::Directions::Right);
    }
}

impl TypeCommand {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self { state }
    }
}
