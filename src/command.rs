#[derive(Debug, PartialEq, Eq, Hash)]
pub enum CursorCommands {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum BufferCommands {
    NewLineBelow,
    Backspace,
    Type(char),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum EditorCommands {
    Quit,
    Render,
    Start,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PaneCommands<'a> {
    BufferUpdate(&'a Vec<String>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Command<'a> {
    Cursor(CursorCommands),
    Buffer(BufferCommands),
    Editor(EditorCommands),
    Pane(PaneCommands<'a>),
}

pub struct CommandBus {
    listeners: Vec<Box<dyn CommandListener>>,
}

pub trait CommandListener {
    fn call(&mut self, command: &Command, id: u16) -> std::io::Result<()>;
}

impl CommandBus {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, listener: Box<dyn CommandListener>) {
        self.listeners.push(listener);
    }

    pub fn dispatch<T>(&mut self, command: Command) -> std::io::Result<()> {
        for listener in self.listeners.iter_mut() {
            listener.call(&command, 0)?;
        }
        Ok(())
    }
}
