# Glyph Roadmap

Here are the features I want to accomplish with the editor. in no particular 
order.

- [ ] LSP integration
    - I want to implement most of the relevant lsp features, such as hovering, 
    and jumping through definitions.
- [ ] Multi Window (tabs)
    - You should be able to open multiple windows, each with their own state
    meaning you can have multiple panes and buffers on that window. and switch
    between windows as needed.
- [ ] Multi panes
    - A window should be able to split vertically or horizontally to add new
    panes as requested.
- [ ] Request save if dirty
    - If there are any number of buffers with dirty state (unsaved). prompt for
    saving before exiting.
- [ ] UI components and workflow
    - This is a though one. I'm still thinking about how to go about this. so
    i'll overlay what I want this system to achieve [here](#ui-components)
- [ ] File Explorer
    - I want to implement a fully capable file explorer for the editor.
    this will be used as a file-tree to be displayed on the side of the file
    as a lot of editors have. but also to behave like NetRw on vim. so you
    can open the current dir on fullscreen and navigate through the project
- [ ] Fuzzy Finder for files
    - I want to make a fuzzy finder for "greping" for files and moving fast
    between them.
- [ ] Diagnostics pane
    - I want to implement some sort of diagnostics pane, to behave mostly like
    quickfix lists, but to be integrated with LSP by default.
- [ ] GUI version.
    - This is one thing that I want to try to implement. But it will make huge
    changes to the current code, refer to [#GUI Version](#gui-version).

## UI Components
I want to be able at some point to enable puglins on the editor, and plugins
should be able to interact with the UI Components system. So it has to be 
flexible enough to be used by external sources and also robust enough for it
to be able to cover every feature.

One of the things that is difficult for me to define is how to setup the layer
in which these components would be rendered. At first it makes sense to render
all of them on a View level. Which would allow for redirecting certain inputs
to the component that should receive them, for scrolling and other actions.

My current way of thinking about how to implement this is that every component
will be a form of dialog, which will have certain flags to determine which
behavior should be redirected to it.

The view would then have possibly a list of components, which are all the 
components that should be drawn to the screen on the next render cycle.

Each dialog could for instance then have a `focusable` flag which would tell
the editor if it should place the handle on it.

The dialog will gain a cursor and motions will be enabled, so we can treat the
dialog just as a regular pane (this means we will probably have to extract the
moving around part from a pane into its own trait, so we can impl Movable, or
something similar)

I want this system to be flexible so we can maybe handle different stuff in
different ways. Lets say for instance an user wants, instead of getting the
current token hover information on a popup, to get it as a pane under the 
current one. It should be able to do that!

## GUI Version
This is a goal for the future. This is not something that I consider to be 
required to achieve in order to have a software that I'm satisfied with. but
rather something that I think will challenge me in writting code to be extended.

Having an GUI version will imply in many changes through the codebase. The ones
that come to my mind right away are that every component that is `core` to the
editor, such as `pane` and `window` have references to stdout, which would have
to be decoupled, and maybe instead the editor should use traits that implement
the functionality and then have a `tui-view` and `gui-view`, which only one of
them would be included for each of the builds.

This is far away and I'm not stressing about this too much by now.
