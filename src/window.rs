use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::pane::Pane;

#[derive(Debug)]
pub struct Window {
    pub panes: HashMap<u16, Arc<Mutex<Pane>>>,
}

impl Window {
    pub fn new(pane: Arc<Mutex<Pane>>) -> Self {
        let pane_lock = pane.lock().unwrap();
        let mut panes = HashMap::new();
        panes.insert(pane_lock.id, Arc::clone(&pane));
        Self { panes }
    }
}
