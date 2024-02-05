mod buffer;
mod command;
mod config;
mod editor;
mod events;
mod pane;
mod view;
mod window;

use editor::Editor;

fn main() -> std::io::Result<()> {
    let file_name = std::env::args().nth(1);
    let mut editor = Editor::new(file_name)?;
    editor.start()?;
    Ok(())
}
