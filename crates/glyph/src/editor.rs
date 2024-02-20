use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use futures::{future::FutureExt, StreamExt};

use crate::buffer::Buffer;
use crate::command::{Command, EditorCommands};
use crate::events::Events;
use crate::lsp::LspClient;
use crate::pane::Pane;
use crate::view::View;
use crate::window::Window;
use crossterm::event::EventStream;

pub struct Editor<'a> {
    events: Events,
    view: View<'a>,
    lsp: &'a LspClient,
}

impl<'a> Editor<'a> {
    pub fn new(file_name: Option<String>, lsp: &'a LspClient) -> anyhow::Result<Self> {
        let buffer = Rc::new(RefCell::new(Buffer::new(1, file_name)?));
        let pane = Pane::new(1, buffer.clone());
        let window = Window::new(1, pane);
        Ok(Self {
            events: Events::new(),
            view: View::new(lsp, window)?,
            lsp,
        })
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.view.handle(Command::Editor(EditorCommands::Start))?;

        let mut stream = EventStream::new();
        let mut client = LspClient::start().await.unwrap();
        client.initialize().await?;

        loop {
            let delay = futures_timer::Delay::new(Duration::from_millis(300)).fuse();
            let event = stream.next().fuse();

            tokio::select! {
                _ = delay => {
                    if let Some((msg, _method)) = client.try_read_message().await? {
                        logger::trace!("[LSP] received message {msg:?}");
                    }
                }
                maybe_event = event => {
                    match maybe_event {
                        Some(Ok(event)) => {
                            if let Some(command) =  self.events.handle(event) {
                                match command {
                                    Command::Editor(EditorCommands::Quit) => {
                                        self.view.handle(command)?;
                                        break
                                    }
                                    _ => self.view.handle(command)?,

                                }
                            }
                        }
                        Some(Err(_)) => (),
                        None => (),
                    }
                }
            }
        }

        Ok(())
    }
}
