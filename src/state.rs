use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{buffer::Buffer, pane::Pane, window::Window};

#[derive(Debug)]
pub struct State {
    pub active_buffer: Option<Arc<Mutex<Buffer>>>,
    pub active_pane: Option<Rc<RefCell<Pane>>>,
    pub active_window: Option<Rc<RefCell<Window>>>,
    pub is_quitting: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            active_buffer: None,
            active_pane: None,
            active_window: None,
            is_quitting: false,
        }
    }

    pub fn set_active_buffer(&mut self, buffer: Arc<Mutex<Buffer>>) {
        self.active_buffer = Some(buffer);
    }

    pub fn set_active_pane(&mut self, pane: Rc<RefCell<Pane>>) {
        self.active_pane = Some(pane);
    }

    pub fn set_active_window(&mut self, window: Rc<RefCell<Window>>) {
        self.active_window = Some(window);
    }
}
