use std::io;
use std::path::PathBuf;

use crossterm::event::{Event, KeyCode, KeyEvent};
use futures::{Stream, StreamExt};
use glyph_config::GlyphConfig;
use glyph_core::editor::{Editor, OpenAction};
use glyph_core::rect::Point;
use glyph_core::syntax::Highlighter;
use glyph_term::backend::Backend;
use glyph_term::layers::editor_layer::EditorLayer;
use glyph_term::renderer::{DrawContext, Renderer};
use glyph_term::terminal::Terminal;

#[derive(Debug)]
pub struct Glyph<'a, B: Backend> {
    terminal: Terminal<B>,
    editor: Editor,
    highlighter: Highlighter,
    renderer: Renderer,
    config: GlyphConfig<'a>,
}

impl<'a, B> Glyph<'a, B>
where
    B: Backend,
{
    pub fn new(backend: B, config: GlyphConfig) -> Glyph<B> {
        let file = std::env::args().nth(1);

        let mut editor = Editor::new(backend.area().expect("couldn't get terminal size"));
        let terminal = Terminal::new(backend);
        let mut renderer = Renderer::new();
        let editor_layer = EditorLayer::new();
        renderer.push_layer(Box::new(editor_layer));

        let document = if let Some(file) = file {
            let content = std::fs::read_to_string(&file).unwrap();
            editor.new_file_with_document(PathBuf::from(file), content, OpenAction::SplitVertical)
        } else {
            editor.new_file(OpenAction::SplitVertical)
        };

        let mut highlighter = Highlighter::new();
        let document = editor.document(&document).unwrap();
        highlighter.add_document(document);

        Glyph {
            editor,
            terminal,
            renderer,
            highlighter,
            config,
        }
    }

    // TODO: make an actual error type
    pub async fn run<S>(&mut self, input_stream: &mut S) -> Result<(), Box<dyn std::error::Error>>
    where
        S: Stream<Item = io::Result<Event>> + Unpin,
    {
        self.terminal.backend.setup()?;

        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            // TODO: restore terminal before panicking
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
            if self.editor.should_close() {
                break Ok(());
            }

            tokio::select! {
                biased;

                Some(event) = input_stream.next() => {
                    if let Ok(Event::Key(KeyEvent { code: KeyCode::Char('q'), .. })) = event {
                        break Ok(());
                    }
                    // TODO: handle the event
                    self.draw_frame()?;
                }

            }
        }
    }

    fn draw_frame(&mut self) -> Result<(), std::io::Error> {
        let mut context = DrawContext {
            editor: &self.editor,
            highlighter: &self.highlighter,
        };
        let buffer = self.terminal.current_buffer();
        self.renderer.draw_frame(buffer, &mut context, self.config);
        self.terminal.flush()?;

        let (pos, kind) = self.renderer.cursor(&self.editor, self.config);
        if let Some(Point { x, y }) = pos {
            self.terminal.backend.set_cursor(x, y, kind)?;
        } else {
            self.terminal.backend.hide_cursor()?;
        }

        Ok(())
    }
}
