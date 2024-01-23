mod editor;
mod event_handler;

use editor::Editor;

fn main() -> std::io::Result<()> {
    let mut editor = Editor::new();
    editor.start()?;
    Ok(())
}
