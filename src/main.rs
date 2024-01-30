mod buffer;
mod editor;
mod pane;
mod view;
mod window;

use editor::Editor;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    let filename = args.nth(1);
    let mut editor = Editor::new();
    editor.start()?;
    Ok(())
}
