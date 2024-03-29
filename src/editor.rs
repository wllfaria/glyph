use std::{
    cell::RefCell,
    io::{stdout, Write},
    rc::Rc,
    sync::mpsc,
    time::Duration,
};

use crossterm::event::EventStream;
use futures::{future::FutureExt, StreamExt};
use serde::{Deserialize, Serialize};

use crate::{
    buffer::Buffer,
    config::{Action, Config, KeyAction},
    events::Events,
    lsp::{IncomingMessage, LspClient},
    pane::Pane,
    theme::Theme,
    tui::{
        layout::{Layout, LayoutUpdate},
        rect::Rect,
        statusline::{Statusline, StatuslineContext},
        Renderable,
    },
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
    window: Window<'a>,
    theme: &'a Theme,

    lsp: LspClient,
    mode: Mode,

    statusline: Statusline,
}

impl<'a> Editor<'a> {
    pub async fn new(
        config: &'a Config,
        theme: &'a Theme,
        lsp: LspClient,
        file_name: Option<String>,
    ) -> anyhow::Result<Self> {
        let buffer = Rc::new(RefCell::new(Buffer::new(1, file_name.clone())?));
        let pane = Pane::new(1, buffer.clone(), theme, config);
        let window = Window::new(1, pane);

        let size = crossterm::terminal::size()?;
        let size = Rect::from(size);

        let mut editor = Self {
            events: Events::new(config),
            window,
            lsp,
            theme,
            mode: Mode::Normal,
            statusline: Statusline::new(Rect::new(size.x, size.bottom() - 2, size.width, 1)),
        };

        editor.start().await?;

        Ok(editor)
    }

    fn setup_terminal(&self) -> anyhow::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            stdout(),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0),
        )?;

        Ok(())
    }

    pub fn cleanup_terminal(&self) -> anyhow::Result<()> {
        crossterm::execute!(stdout(), crossterm::terminal::LeaveAlternateScreen);
        crossterm::terminal::disable_raw_mode()?;

        Ok(())
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.setup_terminal()?;

        let mut stream = EventStream::new();
        self.lsp.initialize().await?;

        self.statusline.render(&StatuslineContext {
            cursor: (1, 1),
            file_name: self
                .window
                .get_active_pane()
                .buffer
                .borrow()
                .file_name
                .clone(),
            mode: self.mode.clone(),
            statusline_style: self.theme.statusline.clone(),
        });

        loop {
            let delay = futures_timer::Delay::new(Duration::from_millis(30));
            let event = stream.next();

            tokio::select! {
                _ = delay => {
                    if let Some(message) = self.lsp.try_read_message().await? {
                        // self.handle_lsp_message(message)?;
                    }
                }
                maybe_event = event => {
                    if let Some(Ok(event)) = maybe_event {
                        if let Some(action) = self.events.handle(&event, &self.mode) {
                            match action {
                                KeyAction::Simple(Action::Quit) => break,
                                _ => self.handle_action(action).await?,
                            }
                        }
                    };
                }
            }

            crossterm::execute!(stdout(), crossterm::cursor::Show)?;
        }

        self.cleanup_terminal()?;

        Ok(())
    }

    async fn handle_action(&mut self, action: KeyAction) -> anyhow::Result<()> {
        match action {
            KeyAction::Simple(Action::EnterMode(mode)) => self.mode = mode,
            KeyAction::Simple(Action::Hover) => {
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
