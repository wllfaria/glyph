mod buffer;
mod commands;
mod editor;
mod keyboard;
mod pane;
mod state;
mod window;

use editor::Editor;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    let filename = args.nth(1);
    let mut editor = Editor::new(filename)?;
    editor.start()?;
    Ok(())
}
