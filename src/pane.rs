use std::sync::{Arc, Mutex};

use crate::buffer::Buffer;

#[derive(Debug)]
pub struct Pane {
    pub id: u16,
    pub buffer: Arc<Mutex<Buffer>>,
}

impl Pane {
    pub fn new(id: u16, buffer: Arc<Mutex<Buffer>>) -> Self {
        Self { id, buffer }
    }
}
