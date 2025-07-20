When the event loop produces an event, it needs to be given to the moving parts
that are relevant.

First, I think it should go into some sort of `KeymapManager`, which will
consume the event and produce an action or command based on it. This will allow
the editor to support keymaps dynamically, since its just a trait

The command should then be passed down to the responsible entity for handling
it, like the buffer manager if it is, for example, a simple character to include
to the current active buffer.

Every iteration of the render loop will end up by calling the renderer with a
new `RenderContext`, which will be a snapshot of the current tree view of the
editor, not really sure if the renderer should calculate what to render or the
editor should give the slice of the buffer content that fits its size.

Essentially this should render white text. But the editor will need some sort of
stacking, or `overlays`, which would be applied on top of the regular text. Like
syntax highlighting, or popups?
