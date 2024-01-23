use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{buffer::Buffer, pane::Pane, window::Window};

pub struct State {
    pub active_buffer: Arc<Mutex<Buffer>>,
    pub active_pane: Rc<RefCell<Pane>>,
    pub active_window: Rc<RefCell<Window>>,
}

impl State {
    pub fn new(
        buffer: Arc<Mutex<Buffer>>,
        pane: Rc<RefCell<Pane>>,
        window: Rc<RefCell<Window>>,
    ) -> Self {
        Self {
            active_buffer: buffer,
            active_pane: pane,
            active_window: window,
        }
    }
}
