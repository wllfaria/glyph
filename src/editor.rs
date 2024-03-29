use std::{cell::RefCell, io::stdout, rc::Rc, sync::mpsc, time::Duration};

use crossterm::event::EventStream;
use futures::{future::FutureExt, StreamExt};
use serde::{Deserialize, Serialize};

use crate::{
    buffer::Buffer,
    config::{Action, Config},
    events::Events,
    lsp::{IncomingMessage, LspClient},
    pane::Pane,
    theme::Theme,
    tui::{layout::Layout, Rect},
    window::Window,
};

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
    lsp: LspClient,
    layout: Layout,
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
        let mode = Mode::Normal;
        let size = crossterm::terminal::size()?;
        let size = Rect::from(size);
        let mut editor = Self {
            events: Events::new(config),
            // view: View::new(config, theme, window, tx, mode.clone())?,
            layout: Layout::new(size),
            lsp,
            mode,
        };

        editor.start(rx).await?;

        Ok(editor)
    }

    pub async fn start(&mut self, rx: mpsc::Receiver<Action>) -> anyhow::Result<()> {
        // self.view.initialize()?;
        crossterm::execute!(
            stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )?;
        self.layout.render()?;

        let mut stream = EventStream::new();
        self.lsp.initialize().await?;

        loop {
            let delay = futures_timer::Delay::new(Duration::from_millis(30));
            let event = stream.next();

            if let Ok(action) = rx.try_recv() {
                match action {
                    Action::Quit => {
                        // self.view.shutdown()?;
                        break;
                    }
                    _ => self.handle_action(action).await?,
                }
            }

            tokio::select! {
                _ = delay => {
                    if let Some(message) = self.lsp.try_read_message().await? {
                        // self.handle_lsp_message(message)?;
                    }
                }
                maybe_event = event => {
                    if let Some(Ok(event)) = maybe_event {
                        if let Some(action) = self.events.handle(&event, &self.mode) {
                            // self.view.handle_action(&action)?;
                        }
                    };
                }
            }
        }

        Ok(())
    }

    async fn handle_action(&mut self, action: Action) -> anyhow::Result<()> {
        match action {
            Action::EnterMode(mode) => self.mode = mode,
            Action::Hover => {
                // TODO: find a better way to grab the file path and information. Maybe
                // have the view give this data instead of querying like this.
                // let pane = self.view.get_active_window().get_active_pane();
                // let cursor = &pane.cursor;
                // let file_name = pane.buffer.borrow().file_name.clone();
                // let row = cursor.row;
                // let col = cursor.col;
                // self.lsp.request_hover(&file_name, row, col).await?;
            }
            _ => (),
        };
        Ok(())
    }

    fn handle_lsp_message(
        &mut self,
        message: (IncomingMessage, Option<String>),
    ) -> anyhow::Result<()> {
        // self.view.handle_lsp_message(message)?;
        Ok(())
    }
}
