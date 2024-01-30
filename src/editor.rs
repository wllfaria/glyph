use std::io::Result;

use crate::view::View;

pub struct Editor {
    is_running: bool,
    view: View,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            is_running: true,
            view: View::new(),
        }
    }

    pub fn start(&mut self) -> Result<()> {
        self.view.initialize();

        while self.is_running {
            self.view.render();
        }

        self.view.shutdown();
        Ok(())
    }
}
