use std::{
    cell::RefCell,
    io::{stdout, Write},
    rc::Rc,
    time::Duration,
};

use crossterm::{event::EventStream, style::Stylize};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::{
    buffer::TextObject,
    config::{Action, Config, KeyAction},
    events::Events,
    frame::Frame,
    lsp::{IncomingMessage, LspClient},
    theme::Theme,
    tui::{
        buffer::{Buffer, FocusableBuffer},
        rect::Rect,
        statusline::{Statusline, StatuslineContext},
        Focusable, Renderable,
    },
};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
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
    buffer: FocusableBuffer<'a>,
    lsp: LspClient,

    theme: &'a Theme,

    area: Rect,
    frames: [Frame; 2],

    mode: Mode,

    statusline: Statusline<'a>,
}

impl<'a> Editor<'a> {
    pub async fn new(
        config: &'a Config,
        theme: &'a Theme,
        lsp: LspClient,
        file_name: Option<String>,
    ) -> anyhow::Result<Self> {
        let size = crossterm::terminal::size()?;
        let size = Rect::from(size);
        let pane_size = size.clone().shrink_bottom(2);

        let text_object = Rc::new(RefCell::new(TextObject::new(1, file_name.clone())?));
        let buffer = Buffer::focusable(1, text_object.clone(), pane_size, config, theme, true);

        let mut editor = Self {
            events: Events::new(config),
            lsp,
            theme,
            buffer,
            mode: Mode::Normal,
            frames: [
                Frame::new(size.width, size.height),
                Frame::new(size.width, size.height),
            ],
            statusline: Statusline::new(Rect::new(size.x, size.bottom() - 2, size.width, 1), theme),
            area: size,
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

    fn draw_cursor(&self) -> anyhow::Result<()> {
        self.buffer.render_cursor(&self.mode)?;

        Ok(())
    }

    fn render_next_frame(&mut self) -> anyhow::Result<()> {
        let frame = &mut self.frames[0];

        self.statusline.render(frame)?;
        self.buffer.render(frame)?;

        self.render_diff()?;
        self.draw_cursor()?;

        stdout().flush()?;

        self.swap_frames();

        Ok(())
    }

    fn fill_frame(&mut self) {
        let frame = &mut self.frames[0];

        for row in 0..self.area.height {
            for col in 0..self.area.width {
                frame.set_cell(col, row, ' ', &self.theme.appearance);
            }
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.setup_terminal()?;
        self.fill_frame();
        self.render_next_frame()?;

        let mut stream = EventStream::new();
        self.lsp.initialize().await?;

        loop {
            let delay = futures_timer::Delay::new(Duration::from_millis(30));
            let event = stream.next();

            tokio::select! {
                _ = delay => {
                    if let Some(message) = self.lsp.try_read_message().await? {
                        self.handle_lsp_message(message)?;
                    }
                }
                maybe_event = event => {
                    if let Some(Ok(event)) = maybe_event {
                        if let Some(action) = self.events.handle(&event, &self.mode) {
                            match action {
                                KeyAction::Simple(Action::EnterMode(_)) => break,
                                KeyAction::Simple(Action::Quit) => break,
                                _ => self.handle_action(action).await?,
                            }
                        }
                    };
                }
            }
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
            KeyAction::Simple(Action::Resize(width, height)) => {
                self.area = Rect::new(0, 0, width, height);
                self.statusline.resize(Rect::new(0, height - 2, width, 1))?;
                self.buffer.resize(Rect::new(0, 0, width, height - 2))?;
            }
            KeyAction::Simple(_) => {
                self.buffer.handle_action(&action, &self.mode)?;
            }
            _ => (),
        };

        self.statusline.update(StatuslineContext {
            cursor: self.buffer.cursor.get_readable_position(),
            file_name: self.buffer.get_file_name(),
            mode: self.mode.clone(),
        });

        self.render_next_frame()?;
        Ok(())
    }

    fn handle_lsp_message(&mut self, _: (IncomingMessage, Option<String>)) -> anyhow::Result<()> {
        Ok(())
    }
}
