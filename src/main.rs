mod buffer;
mod command;
mod cursor;
mod editor;
mod events;
mod pane;
mod window;

use editor::Editor;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    let file_name = args.nth(1);
    let mut editor = Editor::new();
    editor.setup(file_name);
    editor.start()?;
    Ok(())
}
