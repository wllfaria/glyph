use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

use crossterm::event::EventStream;
use futures::{future::FutureExt, StreamExt};
use serde::{Deserialize, Serialize};

use crate::buffer::Buffer;
use crate::config::{Action, Config, KeyAction};
use crate::events::Events;
use crate::lsp::LspClient;
use crate::pane::Pane;
use crate::theme::Theme;
use crate::view::View;
use crate::window::Window;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Search,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Search => f.write_str("SEARCH"),
            Self::Insert => f.write_str("INSERT"),
            Self::Normal => f.write_str("NORMAL"),
            Self::Command => f.write_str("COMMAND"),
        }
    }
}

pub struct Editor<'a> {
    events: Events<'a>,
    view: View<'a>,
    lsp: LspClient,
    config: &'a Config,
    theme: &'a Theme,
    mode: Mode,
}

impl<'a> Editor<'a> {
    pub async fn new(
        config: &'a Config,
        theme: &'a Theme,
        lsp: LspClient,
        file_name: Option<String>,
    ) -> anyhow::Result<Self> {
        let buffer = Rc::new(RefCell::new(Buffer::new(1, file_name)?));
        let pane = Pane::new(1, buffer.clone(), theme, config);
        let window = Window::new(1, pane);
        let (tx, rx) = mpsc::channel::<Action>();
        let mut editor = Self {
            events: Events::new(config),
            view: View::new(config, theme, window, tx)?,
            theme,
            config,
            lsp,
            mode: Mode::Normal,
        };

        editor.start(rx).await?;

        Ok(editor)
    }

    pub async fn start(&mut self, rx: mpsc::Receiver<Action>) -> anyhow::Result<()> {
        self.view.initialize(&self.mode)?;

        let mut stream = EventStream::new();
        self.lsp.initialize().await?;

        loop {
            let delay = futures_timer::Delay::new(Duration::from_millis(300)).fuse();
            let event = stream.next().fuse();

            if let Ok(action) = rx.try_recv() {
                match action {
                    Action::Quit => {
                        self.view.shutdown()?;
                        break;
                    }
                    _ => (),
                }
            }

            tokio::select! {
                _ = delay => {
                    if let Some((msg, _method)) = self.lsp.try_read_message().await? {
                        tracing::trace!("[LSP] received message {msg:?}");
                    }
                }
                maybe_event = event => {
                    if let Some(Ok(event)) = maybe_event {
                        if let Some(action) = self.events.handle(&event, &self.mode) {
                            match action {
                                KeyAction::Simple(Action::Quit) => {
                                    self.view.shutdown()?;
                                    break
                                }
                                KeyAction::Simple(Action::EnterMode(Mode::Normal)) => {
                                    self.mode = Mode::Normal;
                                    self.view.handle_action(&action, &mut self.mode)?;
                                }
                                KeyAction::Simple(Action::EnterMode(Mode::Insert)) => {
                                    self.mode = Mode::Insert;
                                    self.view.handle_action(&action, &mut self.mode)?;
                                }
                                KeyAction::Simple(Action::EnterMode(Mode::Command)) => {
                                    self.mode = Mode::Command;
                                    self.view.handle_action(&action, &mut self.mode)?;
                                }
                                _ => self.view.handle_action(&action, &mut self.mode)?,

                            }
                        }
                    };
                }
            }
        }

        Ok(())
    }
}
