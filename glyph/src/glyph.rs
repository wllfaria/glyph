use std::io;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::Duration;

use futures::{stream, Stream, StreamExt};
use glyph_config::GlyphConfig;
use glyph_core::editor::{Editor, OpenAction};
use glyph_core::rect::Point;
use glyph_term::backend::Backend;
use glyph_term::layers::editor_layer::EditorLayer;
use glyph_term::renderer::{DrawContext, Renderer};
use glyph_term::terminal::Terminal;

#[derive(Debug)]
pub struct Glyph<'a, B: Backend> {
    terminal: Terminal<B>,
    editor: Editor,
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

        if let Some(file) = file {
            let content = std::fs::read_to_string(&file).unwrap();
            editor.new_file_with_document(PathBuf::from(file), content, OpenAction::SplitVertical);
        } else {
            editor.new_file(OpenAction::SplitVertical);
        }

        Glyph {
            editor,
            terminal,
            renderer,
            config,
        }
    }

    // TODO: make an actual error type
    pub async fn run<S>(&mut self, input_stream: &mut S) -> Result<(), Box<dyn std::error::Error>>
    where
        S: Stream<Item = io::Result<crossterm::event::Event>> + Unpin,
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
        S: Stream<Item = io::Result<crossterm::event::Event>> + Unpin,
    {
        loop {
            let start = std::time::Instant::now();
            if self.editor.should_close() {
                break Ok(());
            }

            tokio::select! {
                biased;

                Some(event) = input_stream.next() => {
                    if let Ok(crossterm::event::Event::Key(crossterm::event::KeyEvent { code: crossterm::event::KeyCode::Char('q'), .. })) = event {
                        break Ok(());
                    }
                    // TODO: handle the event
                    self.draw_frame()?;
                }
            }

            let frame_time = start.elapsed();
            tracing::debug!("{frame_time:?}");
        }
    }

    fn draw_frame(&mut self) -> Result<(), std::io::Error> {
        let mut context = DrawContext { editor: &self.editor };
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

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = crossterm::event::Event>>> {
        Box::pin(stream::unfold((), |_| async {
            if crossterm::event::poll(Duration::from_millis(1)).unwrap() {
                if let Ok(ev) = crossterm::event::read() {
                    return Some((ev, ()));
                }
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
            None
        }))
    }
}
