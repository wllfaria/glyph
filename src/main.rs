mod buffer;
mod editor;
mod event_handler;
mod pane;
mod window;

use editor::Editor;

fn main() -> std::io::Result<()> {
    let mut editor = Editor::new();
    editor.start()?;
    Ok(())
}
