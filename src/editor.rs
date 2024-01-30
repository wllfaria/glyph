use crossterm::{terminal, QueueableCommand};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Result, Write};
use std::rc::Rc;

use crate::buffer::Buffer;
use crate::command::{Command, CommandBus, EditorCommands};
use crate::events::Events;
use crate::pane::Pane;
use crate::window::Window;

pub struct Editor {
    should_quit: bool,
    command_bus: Rc<RefCell<CommandBus>>,
    event_handler: Events,
    windows: HashMap<u16, Rc<RefCell<Window>>>,
    panes: HashMap<u16, Rc<RefCell<Pane>>>,
    buffers: HashMap<u16, Rc<RefCell<Buffer>>>,
    stdout: std::io::Stdout,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            panes: HashMap::new(),
            buffers: HashMap::new(),
            should_quit: false,
            event_handler: Events::new(),
            command_bus: Rc::new(RefCell::new(CommandBus::new())),
            stdout: std::io::stdout(),
        }
    }

    pub fn setup(&mut self, file_name: Option<String>) {
        let command_bus = self.command_bus.clone();
        let buffer = Rc::new(RefCell::new(Buffer::new(1, file_name, command_bus.clone())));
        let pane = Rc::new(RefCell::new(Pane::new(1, command_bus.clone())));
        let window = Rc::new(RefCell::new(Window::new(
            1,
            command_bus.clone(),
            pane.clone(),
        )));

        window.borrow_mut().setup(window.clone());
        pane.borrow_mut().setup(pane.clone());
        buffer.borrow_mut().setup(buffer.clone());

        self.windows.insert(window.borrow().id, window.clone());
        self.panes.insert(pane.borrow().id, pane.clone());
        self.buffers.insert(buffer.borrow().id, buffer.clone());
    }

    fn clear_screen(&mut self) -> Result<()> {
        self.stdout
            .queue(crossterm::cursor::MoveTo(0, 0))?
            .queue(crossterm::terminal::Clear(
                crossterm::terminal::ClearType::All,
            ))?;
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;

        self.clear_screen()?;

        self.command_bus
            .borrow_mut()
            .dispatch::<EditorCommands>(Command::Editor(EditorCommands::Start))?;

        while !self.should_quit {
            if let Some(command) = self.event_handler.poll_events()? {
                match command {
                    Command::Editor(EditorCommands::Quit) => self.should_quit = true,
                    _ => self.command_bus.borrow_mut().dispatch::<Command>(command)?,
                }
            }
            self.command_bus
                .borrow_mut()
                .dispatch::<EditorCommands>(Command::Editor(EditorCommands::Render))?;
            self.stdout.flush()?
        }

        self.clear_screen()?;
        self.stdout.flush()?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}
