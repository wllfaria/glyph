use std::io;
use std::path::PathBuf;

use futures::{Stream, StreamExt};
use glyph_term::backend::Backend;
use glyph_term::terminal::Terminal;

use crate::editor::{Editor, OpenAction};
use crate::layers::editor_layer::EditorLayer;
use crate::renderer::{DrawContext, Renderer};

#[derive(Debug)]
pub struct Glyph<B: Backend> {
    terminal: Terminal<B>,
    editor: Editor,
    renderer: Renderer,
}

impl<B> Glyph<B>
where
    B: Backend,
{
    pub fn new(backend: B) -> Glyph<B> {
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

        self.draw_frame();
        self.event_loop(input_stream).await;

        self.terminal.backend.restore()?;
        Ok(())
    }

    async fn event_loop<S>(&mut self, input_stream: &mut S)
    where
        S: Stream<Item = io::Result<crossterm::event::Event>> + Unpin,
    {
        loop {
            if self.editor.should_close() {
                break;
            }

            tokio::select! {
                biased;

                Some(event) = input_stream.next() => {
                    if let Ok(crossterm::event::Event::Key(crossterm::event::KeyEvent { code: crossterm::event::KeyCode::Char('q'), .. })) = event {
                        break;
                    }
                    // TODO: handle the event
                    self.draw_frame();
                }
            }
        }
    }

    fn draw_frame(&mut self) {
        let mut context = DrawContext { editor: &self.editor };
        self.renderer.draw_frame(self.terminal.current_buffer(), &mut context);
        _ = self.terminal.flush();
    }
}
