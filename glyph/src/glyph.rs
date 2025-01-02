use std::collections::BTreeMap;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::sync::Arc;

use crossterm::event::Event;
use futures::{Stream, StreamExt};
use glyph_config::dirs::DIRS;
use glyph_config::Config;
use glyph_core::cursor::Cursor;
use glyph_core::editor::{Editor, EventResult, OpenAction};
use glyph_core::rect::Point;
use glyph_core::syntax::Highlighter;
use glyph_core::window::WindowId;
use glyph_runtime::{GlyphContext, RuntimeMessage};
use glyph_term::backend::{Backend, CrosstermBackend};
use glyph_term::layers::editor_layer::EditorLayer;
use glyph_term::renderer::{Context, Renderer};
use glyph_term::terminal::Terminal;
use mlua::Lua;
use parking_lot::RwLock;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

#[derive(Debug)]
pub struct Glyph<'a, B>
where
    B: Backend,
{
    terminal: Terminal<B>,
    editor: Arc<RwLock<Editor>>,
    cursors: Arc<RwLock<BTreeMap<WindowId, Cursor>>>,
    highlighter: Highlighter,
    renderer: Renderer,
    config: Config<'a>,
    runtime: Lua,
    runtime_receiver: ReceiverStream<RuntimeMessage<'static>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum ControlFlow {
    Break,
    Continue,
}

#[derive(Debug)]
struct ReceiverStream<T>(UnboundedReceiver<T>);

impl<T> Stream for ReceiverStream<T> {
    type Item = T;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_recv(cx)
    }
}

impl<'a, B> Glyph<'a, B>
where
    B: Backend,
{
    pub fn new(backend: B) -> glyph_runtime::error::Result<Glyph<'a, B>> {
        let file = std::env::args().nth(1);

        let mut editor = Editor::new(backend.area().expect("couldn't get terminal size"));
        let terminal = Terminal::new(backend);
        let mut renderer = Renderer::new();
        let editor_layer = EditorLayer::new();
        renderer.push_layer(Box::new(editor_layer));

        let (window, document) = if let Some(file) = file {
            let content = std::fs::read_to_string(&file).expect("only files are supported");
            editor.new_file_with_document(PathBuf::from(file), content, OpenAction::SplitVertical)
        } else {
            editor.new_file(OpenAction::SplitVertical)
        };

        let mut highlighter = Highlighter::new();
        let document = editor.document(document);
        highlighter.add_document(document);

        let mut cursors = BTreeMap::default();
        cursors.insert(window, Cursor::default());

        let editor = Arc::new(RwLock::new(editor));
        let cursors = Arc::new(RwLock::new(cursors));

        let glyph_context = Arc::new(RwLock::new(GlyphContext {
            editor: editor.clone(),
            cursors: cursors.clone(),
        }));
        let (runtime_sender, mut runtime_receiver) = unbounded_channel();
        let runtime = glyph_runtime::setup_lua_runtime(
            DIRS.get().unwrap().config(),
            runtime_sender.clone(),
            glyph_context.clone(),
        )?;
        let config = glyph_config::Config::load(&runtime, &mut runtime_receiver)?;
        let runtime_receiver = ReceiverStream(runtime_receiver);

        Ok(Glyph {
            editor,
            terminal,
            renderer,
            highlighter,
            config,
            cursors,
            runtime,
            runtime_receiver,
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

            let control_flow = tokio::select! {
                biased;

                Some(event) = input_stream.next() => {
                    self.handle_event(event?)?;
                    self.draw_frame()?;
                    ControlFlow::Continue
                }
                Some(message) = self.runtime_receiver.next() => self.handle_runtime_message(message)?,
            };

            if control_flow == ControlFlow::Break {
                break Ok(());
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> Result<Option<EventResult>, io::Error> {
        if let Event::Resize(width, height) = event {
            self.terminal.resize((height, width).into());
        };

        let mut context = Context {
            editor: self.editor.clone(),
            highlighter: &mut self.highlighter,
            runtime: &self.runtime,
            cursors: self.cursors.clone(),
        };
        self.renderer.handle_event(&event, &mut context, &self.config)
    }

    fn handle_runtime_message(&mut self, message: RuntimeMessage) -> Result<ControlFlow, io::Error> {
        match message {
            RuntimeMessage::UpdateHighlightGroup(_, _) => todo!(),
            RuntimeMessage::SetKeymap(_) => todo!(),
            RuntimeMessage::UserCommandCreate(_, _) => todo!(),
            RuntimeMessage::Error(_) => todo!(),
            RuntimeMessage::Quit(options) => match (options.force, options.all) {
                (true, true) => return Ok(ControlFlow::Break),
                (true, false) => todo!(),
                (false, true) => {}
                (false, false) => {
                    if self.editor.write().close_active_window() {
                        return Ok(ControlFlow::Break);
                    }
                }
            },
            RuntimeMessage::Write(options) => match (options.force, options.all) {
                (true, true) => {}
                (true, false) => {}
                (false, true) => {}
                (false, false) => {
                    let mut editor = self.editor.write();
                    let tab = editor.focused_tab();
                    let window = tab.tree.focus();
                    let window = tab.tree.window(window);
                    let document = window.document;
                    editor.write_document(document);
                    drop(editor);
                    self.draw_frame()?;
                }
            },
            RuntimeMessage::OpenFile(filename) => {
                let mut editor = self.editor.write();
                if filename.is_empty() {
                    editor.new_file(OpenAction::Replace);
                    return Ok(ControlFlow::Continue);
                }
                let text = std::fs::read_to_string(&filename).expect("TODO: handle this error");
                let (_, document) = editor.new_file_with_document(filename, text, OpenAction::Replace);
                let document = editor.document(document);
                self.highlighter.add_document(document);

                drop(editor);
                self.draw_frame()?;
            }
        };

        Ok(ControlFlow::Continue)
    }

    fn draw_frame(&mut self) -> Result<(), std::io::Error> {
        let mut context = Context {
            editor: self.editor.clone(),
            highlighter: &mut self.highlighter,
            runtime: &self.runtime,
            cursors: self.cursors.clone(),
        };
        let buffer = self.terminal.current_buffer();
        self.renderer.draw_frame(buffer, &mut context, &self.config);
        self.terminal.flush(&self.config)?;
        self.terminal.swap_buffers();

        let (pos, kind) = self.renderer.cursor(&mut context, &self.config);
        if let Some(Point { x, y }) = pos {
            self.terminal.backend.set_cursor(x, y, kind)?;
        } else {
            self.terminal.backend.hide_cursor()?;
        }

        Ok(())
    }
}
