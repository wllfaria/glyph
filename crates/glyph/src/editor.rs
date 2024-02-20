use std::time::Duration;

use futures::{future::FutureExt, StreamExt};

use crate::command::{Command, EditorCommands};
use crate::events::Events;
use crate::lsp::{IncomingMessage, LspClient};
use crate::view::View;
use crossterm::event::EventStream;

pub struct Editor {
    events: Events,
    view: View,
}

impl Editor {
    pub fn new(file_name: Option<String>) -> anyhow::Result<Self> {
        Ok(Self {
            events: Events::new(),
            view: View::new(file_name)?,
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
                    if let Some((_msg, _method)) = client.try_read_message().await? { }
                }
                maybe_event = event => {
                    match maybe_event {
                        Some(Ok(event)) => {
                            match self.events.handle(event) {
                                Some(command) => match command {
                                    Command::Editor(EditorCommands::Quit) => {
                                        self.view.handle(command)?;
                                        break
                                    }
                                    _ => self.view.handle(command)?,

                                }
                                None => (),
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
