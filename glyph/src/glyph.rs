use std::io;

use futures::{Stream, StreamExt};
use glyph_tui::backend::Backend;

use crate::editor::Editor;

#[derive(Debug)]
pub struct Glyph<B: Backend> {
    backend: B,
    editor: Editor,
}

impl<B> Glyph<B>
where
    B: Backend,
{
    pub fn new(backend: B) -> Glyph<B> {
        Glyph {
            backend,
            editor: Editor::new(),
        }
    }

    // TODO: make an actual error type
    pub async fn run<S>(&mut self, input_stream: &mut S) -> Result<(), Box<dyn std::error::Error>>
    where
        S: Stream<Item = io::Result<crossterm::event::Event>> + Unpin,
    {
        self.backend.setup()?;

        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            // TODO: restore terminal before panicking
            hook(info);
        }));

        self.event_loop(input_stream).await;

        self.backend.restore()?;
        Ok(())
    }

    async fn event_loop<S>(&mut self, input_stream: &mut S)
    where
        S: Stream<Item = io::Result<crossterm::event::Event>> + Unpin,
    {
        loop {
            if self.editor.should_close {
                break;
            }

            tokio::select! {
                biased;

                Some(_event) = input_stream.next() => {
                    // TODO: handle the event
                    self.editor.should_close = true;
                }
            }
        }
    }
}
