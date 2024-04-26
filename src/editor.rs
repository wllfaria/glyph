use std::{
    cell::RefCell,
    collections::HashMap,
    io::{stdout, Write},
    path::{Path, PathBuf},
    rc::Rc,
    time::Duration,
};

use crossterm::{event::EventStream, execute, style::Stylize};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::{
    buffer::TextObject,
    config::{Action, Config, KeyAction},
    cursor::Cursor,
    events::Events,
    frame::{cell::Cell, Frame},
    lsp::{IncomingMessage, LspClient},
    theme::Theme,
    tui::{
        buffer::{Buffer, WithCursor},
        commandline::{CommandKind, Commandline},
        create_popup,
        rect::Rect,
        statusline::{Statusline, StatuslineContext},
        Focusable, Renderable,
    },
};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Command,
    Search,
}

impl Default for &Mode {
    fn default() -> Self {
        &Mode::Normal
    }
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
    buffers: HashMap<usize, Buffer<'a, WithCursor>>,
    current_buffer: usize,
    next_buffer: usize,

    lsp: LspClient,

    theme: &'a Theme,
    config: &'a Config,

    area: Rect,
    frames: [Frame; 2],

    mode: Mode,

    statusline: Statusline<'a>,
    commandline: Commandline<'a>,
    popup: Option<Buffer<'a>>,

    action_rx: UnboundedReceiver<Action>,
}

impl<'a> Editor<'a> {
    pub async fn new(
        config: &'a Config,
        theme: &'a Theme,
        lsp: LspClient,
        file_name: Option<String>,
    ) -> anyhow::Result<Self> {
        let area = crossterm::terminal::size()?;
        let area = Rect::from(area);
        let pane_size = area.clone().shrink_bottom(2);
        let statusline_size = Rect::new(area.x, area.bottom() - 2, area.width, 1);
        let commandline_size = Rect::new(area.x, area.bottom() - 1, area.width, 1);

        let text_object = Rc::new(RefCell::new(TextObject::new(1, file_name.clone())?));
        let buffer =
            Buffer::new(1, text_object.clone(), pane_size, config, theme, false).with_cursor();

        let (action_tx, action_rx) = unbounded_channel::<Action>();
        let mut buffers = HashMap::new();
        buffers.insert(1, buffer);

        let mut editor = Self {
            events: Events::new(config),
            lsp,
            theme,
            mode: Mode::Normal,
            frames: [
                Frame::new(area.width, area.height),
                Frame::new(area.width, area.height),
            ],
            statusline: Statusline::new(statusline_size, theme),
            commandline: Commandline::new(commandline_size, theme),
            area,
            config,
            popup: None,
            action_rx,
            buffers,
            current_buffer: 1,
            next_buffer: 2,
        };

        editor.start(action_tx.clone()).await?;

        Ok(editor)
    }

    fn setup_terminal(&self) -> anyhow::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            stdout(),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        )?;

        Ok(())
    }

    pub fn cleanup_terminal(&self) -> anyhow::Result<()> {
        crossterm::execute!(stdout(), crossterm::terminal::LeaveAlternateScreen)?;
        crossterm::terminal::disable_raw_mode()?;

        Ok(())
    }

    fn swap_frames(&mut self) {
        self.frames.swap(0, 1);
        self.frames[0].clear();
    }

    fn render_diff(&mut self) -> anyhow::Result<()> {
        let (current, last) = (&self.frames[0], &self.frames[1]);
        let diffs = current.diff(last);

        for diff in diffs {
            let x = self.area.x + diff.col;
            let y = self.area.y + diff.row;

            let mut cell = diff.cell.c.stylize();

            if let Some(bg) = diff.cell.style.bg {
                cell = cell.on(bg);
            } else {
                cell = cell.on(self.theme.appearance.bg.unwrap());
            }

            if let Some(fg) = diff.cell.style.fg {
                cell = cell.with(fg);
            } else {
                cell = cell.with(self.theme.appearance.fg.unwrap());
            }

            if let Some(true) = diff.cell.style.bold {
                cell = cell.bold();
            }

            if let Some(true) = diff.cell.style.italic {
                cell = cell.italic();
            }

            crossterm::queue!(
                stdout(),
                crossterm::cursor::MoveTo(x, y),
                crossterm::style::PrintStyledContent(cell)
            )?;
        }

        Ok(())
    }

    fn render_next_frame(&mut self) -> anyhow::Result<()> {
        tracing::trace!("[Editor] rendering next frame");
        execute!(stdout(), crossterm::cursor::Hide)?;

        let frame = &mut self.frames[0];

        self.statusline.render(frame)?;
        self.commandline.render(frame)?;

        self.buffers
            .get_mut(&self.current_buffer)
            .unwrap()
            .render(frame)?;

        if let Some(popup) = &mut self.popup {
            popup.render(frame)?;
        }

        self.render_diff()?;

        if self.mode.eq(&Mode::Command) {
            self.commandline.render_cursor()?;
        } else {
            self.buffers
                .get(&self.current_buffer)
                .unwrap()
                .render_cursor(&self.mode)?;
        }

        execute!(stdout(), crossterm::cursor::Show)?;
        stdout().flush()?;

        self.swap_frames();
        Ok(())
    }

    fn fill_frame(&mut self) {
        self.frames[0]
            .cells
            .iter_mut()
            .for_each(|cell| *cell = Cell::new(' ', self.theme.appearance));
    }

    fn open_buffer<S>(&mut self, path: S) -> anyhow::Result<()>
    where
        S: AsRef<Path>,
    {
        let text_object = Rc::new(RefCell::new(TextObject::new(1, Some(path))?));
        let pane_size = self.area.clone().shrink_bottom(2);
        let buffer = Buffer::new(
            self.next_buffer,
            text_object.clone(),
            pane_size,
            self.config,
            self.theme,
            false,
        )
        .with_cursor();
        self.buffers.insert(self.next_buffer, buffer);
        self.current_buffer = self.next_buffer;
        self.next_buffer += 1;

        Ok(())
    }

    pub async fn start(&mut self, action_tx: UnboundedSender<Action>) -> anyhow::Result<()> {
        self.setup_terminal()?;
        self.fill_frame();
        self.render_next_frame()?;

        let mut stream = EventStream::new();
        self.lsp.initialize().await?;

        loop {
            let delay = futures_timer::Delay::new(Duration::from_millis(30));
            let event = stream.next();

            match self.action_rx.try_recv() {
                Ok(Action::Quit) => break,
                Ok(Action::LoadFile(path)) => {
                    self.open_buffer(path)?;
                    self.render_next_frame()?;
                }
                _ => {}
            };

            tokio::select! {
                _ = delay => {
                    if let Some(message) = self.lsp.try_read_message().await? {
                        self.handle_lsp_message(message)?;
                    }
                }
                maybe_event = event => {
                    if let Some(Ok(event)) = maybe_event {
                        if let Some(action) = self.events.handle(&event, &self.mode) {
                            self.popup = None;
                            self.handle_action(action, action_tx.clone()).await?;
                        }
                    };
                }
            }
        }

        self.cleanup_terminal()?;

        Ok(())
    }

    async fn handle_action(
        &mut self,
        action: KeyAction,
        action_tx: UnboundedSender<Action>,
    ) -> anyhow::Result<()> {
        match action {
            KeyAction::Simple(Action::EnterMode(Mode::Command)) => {
                self.commandline.update_kind(CommandKind::Command);
                self.mode = Mode::Command;
            }
            KeyAction::Simple(Action::EnterMode(mode)) => self.mode = mode,
            KeyAction::Simple(Action::Hover) => {
                let Cursor { col, row, .. } =
                    self.buffers.get_mut(&self.current_buffer).unwrap().cursor;
                let file_name = self
                    .buffers
                    .get_mut(&self.current_buffer)
                    .unwrap()
                    .text_object
                    .borrow()
                    .file_name
                    .clone();
                self.lsp.request_hover(&file_name, row, col).await?;
            }
            KeyAction::Simple(Action::Resize(width, height)) => {
                self.area = Rect::new(0, 0, width, height);
                self.statusline.resize(Rect::new(0, height - 2, width, 1))?;
                self.buffers
                    .get_mut(&self.current_buffer)
                    .unwrap()
                    .resize(Rect::new(0, 0, width, height - 2))?;
            }
            KeyAction::Simple(Action::ExecuteCommand) => {
                self.handle_user_command(action_tx.clone())?
            }
            KeyAction::Simple(Action::InsertCommand(_)) => {
                self.commandline.handle_action(&action);
            }
            KeyAction::Simple(Action::DeletePreviousChar) => match self.mode {
                Mode::Command => {
                    if let Some(Action::EnterMode(mode)) = self.commandline.handle_action(&action) {
                        self.mode = mode;
                    }
                }
                _ => self
                    .buffers
                    .get_mut(&self.current_buffer)
                    .unwrap()
                    .handle_action(&action, &self.mode)?,
            },
            KeyAction::Simple(_) => {
                self.buffers
                    .get_mut(&self.current_buffer)
                    .unwrap()
                    .handle_action(&action, &self.mode)?;
            }
            _ => (),
        };

        self.statusline.update(StatuslineContext {
            cursor: self
                .buffers
                .get(&self.current_buffer)
                .unwrap()
                .cursor
                .get_readable_position(),
            file_name: self
                .buffers
                .get(&self.current_buffer)
                .unwrap()
                .get_file_name(),
            mode: self.mode.clone(),
        });

        self.render_next_frame()?;

        Ok(())
    }

    fn handle_user_command(&mut self, action_tx: UnboundedSender<Action>) -> anyhow::Result<()> {
        let command = self.commandline.command();

        // TODO: change this to get available commands from a map instead of matching on a raw &str
        match command.chars().nth(0) {
            Some('q') => action_tx.send(Action::Quit)?,
            Some('e') => {
                if let Some((_, file_path)) = command.split_once(' ') {
                    let cwd = std::env::current_dir()?;
                    let file_path = cwd.join(PathBuf::from(file_path));
                    if file_path.exists() {
                        action_tx.send(Action::LoadFile(file_path))?;
                    }
                }
            }
            _ => {}
        };

        Ok(())
    }

    fn handle_lsp_message(
        &mut self,
        message: (IncomingMessage, Option<String>),
    ) -> anyhow::Result<()> {
        if let Some(method) = message.1 {
            if method.as_str() == "textDocument/hover" {
                let message = message.0;
                if let IncomingMessage::Message(message) = message {
                    let result = match message.result {
                        serde_json::Value::Array(ref array) => array[0].as_object().unwrap(),
                        serde_json::Value::Object(ref object) => object,
                        _ => return Ok(()),
                    };
                    if let Some(contents) = result.get("contents") {
                        if let Some(contents) = contents.as_object() {
                            if let Some(serde_json::Value::String(value)) = contents.get("value") {
                                let buffer = create_popup(
                                    &self.area,
                                    self.buffers
                                        .get(&self.current_buffer)
                                        .unwrap()
                                        .gutter
                                        .as_ref()
                                        .map(|g| g.width())
                                        .unwrap_or(0),
                                    value.clone(),
                                    &self.buffers.get(&self.current_buffer).unwrap().cursor,
                                    self.config,
                                    self.theme,
                                );
                                self.popup = Some(buffer);
                                self.render_next_frame()?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
