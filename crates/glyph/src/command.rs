#[derive(Debug)]
pub enum WindowCommands {
    SplitVertical,
}

#[derive(Debug)]
pub enum EditorCommands {
    Quit,
    Start,
    SecondElapsed,
}

#[derive(Debug)]
pub enum PaneCommands {}

#[derive(Debug)]
pub enum CursorCommands {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
}

#[derive(Debug)]
pub enum BufferCommands {
    NewLineBelow,
    Backspace,
    Type(char),
    Save,
}

#[derive(Debug)]
pub enum Command {
    Window(WindowCommands),
    Buffer(BufferCommands),
    Editor(EditorCommands),
    Cursor(CursorCommands),
    Pane(PaneCommands),
}
