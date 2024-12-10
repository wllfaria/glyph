use std::collections::BTreeMap;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::sync::Arc;

use crossterm::event::{Event, KeyCode, KeyEvent};
use futures::{Stream, StreamExt};
use glyph_config::Config;
use glyph_core::cursor::Cursor;
use glyph_core::editor::{Editor, EventResult, OpenAction};
use glyph_core::rect::Point;
use glyph_core::syntax::Highlighter;
use glyph_core::window::WindowId;
use glyph_runtime::{setup_post_startup_apis, GlyphContext, RuntimeMessage};
use glyph_term::backend::{Backend, CrosstermBackend};
use glyph_term::layers::editor_layer::EditorLayer;
use glyph_term::renderer::{Context, Renderer};
use glyph_term::terminal::Terminal;
use mlua::Lua;
use parking_lot::RwLock;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
pub struct Glyph<'a, B>
where
    B: Backend,
{
    terminal: Terminal<B>,
    editor: Arc<RwLock<Editor>>,
    cursors: BTreeMap<WindowId, Cursor>,
    highlighter: Highlighter,
    renderer: Renderer,
    config: &'a mut Config<'a>,
    runtime: Lua,
}

impl<'a, B> Glyph<'a, B>
where
    B: Backend,
{
    pub fn new(
        backend: B,
        runtime: Lua,
        runtime_sender: UnboundedSender<RuntimeMessage<'static>>,
        _runtime_receiver: UnboundedReceiver<RuntimeMessage<'static>>,
        config: &'a mut Config<'a>,
    ) -> glyph_runtime::error::Result<Glyph<'a, B>> {
        let file = std::env::args().nth(1);

        let mut editor = Editor::new(backend.area().expect("couldn't get terminal size"));
        let terminal = Terminal::new(backend);
        let mut renderer = Renderer::new();
        let editor_layer = EditorLayer::new();
        renderer.push_layer(Box::new(editor_layer));

        let (window, document) = if let Some(file) = file {
            let content = std::fs::read_to_string(&file).unwrap();
            editor.new_file_with_document(PathBuf::from(file), content, OpenAction::SplitVertical)
        } else {
            editor.new_file(OpenAction::SplitVertical)
        };

        let mut highlighter = Highlighter::new();
        let document = editor.document(&document);
        highlighter.add_document(document);

        let mut cursors = BTreeMap::default();
        let cursor = Cursor::default();
        cursors.insert(window, cursor);

        let editor = Arc::new(RwLock::new(editor));

        let glyph_context = GlyphContext { editor: editor.clone() };

        let statusline = setup_post_startup_apis(&runtime, runtime_sender, glyph_context)?;
        println!("{statusline:?}");
        config.statusline = statusline;

        Ok(Glyph {
            editor,
            terminal,
            renderer,
            highlighter,
            config,
            cursors,
            runtime,
        })
    }

    // TODO: make an actual error type
    pub async fn run<S>(&mut self, input_stream: &mut S) -> Result<(), Box<dyn std::error::Error>>
    where
        S: Stream<Item = io::Result<Event>> + Unpin,
    {
        self.terminal.backend.setup()?;

        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            _ = CrosstermBackend::<Stdout>::force_restore();
            hook(info);
        }));

        self.draw_frame()?;
        self.event_loop(input_stream).await?;

        self.terminal.backend.restore()?;
        Ok(())
    }

    async fn event_loop<S>(&mut self, input_stream: &mut S) -> Result<(), std::io::Error>
    where
        S: Stream<Item = io::Result<Event>> + Unpin,
    {
        loop {
            if self.editor.read().should_close() {
                break Ok(());
            }

            tokio::select! {
                biased;

                Some(event) = input_stream.next() => {
                    if let Ok(Event::Key(KeyEvent { code: KeyCode::Char('q'), .. })) = event {
                        break Ok(())
                    };
                    self.handle_event(event?)?;
                    self.draw_frame()?;
                }

            }
        }
    }

    fn handle_event(&mut self, event: Event) -> Result<Option<EventResult>, io::Error> {
        let mut context = Context {
            editor: self.editor.clone(),
            highlighter: &mut self.highlighter,
            cursors: &mut self.cursors,
        };
        self.renderer.handle_event(&event, &mut context, self.config)
    }

    fn draw_frame(&mut self) -> Result<(), std::io::Error> {
        let mut context = Context {
            editor: self.editor.clone(),
            highlighter: &mut self.highlighter,
            cursors: &mut self.cursors,
        };
        let buffer = self.terminal.current_buffer();
        self.renderer.draw_frame(buffer, &mut context, self.config);
        self.terminal.flush(self.config)?;
        self.terminal.swap_buffers();

        let (pos, kind) = self.renderer.cursor(&mut context, self.config);
        if let Some(Point { x, y }) = pos {
            self.terminal.backend.set_cursor(x, y, kind)?;
        } else {
            self.terminal.backend.hide_cursor()?;
        }

        Ok(())
    }
}
