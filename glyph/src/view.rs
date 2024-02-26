use std::collections::HashMap;
use std::io::{stdout, Result, Stdout, Write};
use std::sync::mpsc;

use crossterm::style::Print;
use crossterm::{cursor, style};
use crossterm::{terminal, QueueableCommand};

use crate::config::{Action, Config, KeyAction};
use crate::editor::Mode;
use crate::lsp::IncomingMessage;
use crate::pane::Position;
use crate::theme::{Style, Theme};
use crate::viewport::{Change, Viewport};
use crate::window::Window;

#[derive(Default, Debug, Copy, Clone)]
pub struct Size {
    pub height: usize,
    pub width: usize,
}

impl From<(u16, u16)> for Size {
    fn from((width, height): (u16, u16)) -> Self {
        Self {
            width: width as usize,
            height: height as usize,
        }
    }
}

pub struct View<'a> {
    active_window: usize,
    windows: HashMap<usize, Window<'a>>,
    size: Size,
    stdout: Stdout,
    config: &'a Config,
    statusline: Viewport,
    commandline: Viewport,
    command: String,
    theme: &'a Theme,
    tx: mpsc::Sender<Action>,
}

impl<'a> View<'a> {
    pub fn new(
        config: &'a Config,
        theme: &'a Theme,
        mut window: Window<'a>,
        tx: mpsc::Sender<Action>,
    ) -> Result<Self> {
        let mut windows = HashMap::new();
        let size = terminal::size()?;

        let id = window.id;
        window.resize((size.0, size.1 - 2).into());
        windows.insert(window.id, window);

        Ok(Self {
            stdout: stdout(),
            size: size.into(),
            active_window: id,
            windows,
            config,
            statusline: Viewport::new(size.0 as usize, 1),
            commandline: Viewport::new(size.0 as usize, 1),
            command: String::new(),
            theme,
            tx,
        })
    }

    pub fn handle_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        let last_statusline = self.statusline.clone();
        let last_commandline = self.commandline.clone();
        let mut statusline = Viewport::new(self.size.width, 1);
        let mut commandline = Viewport::new(self.size.width, 1);
        let active_window = self.windows.get_mut(&self.active_window).unwrap();
        match action {
            KeyAction::Simple(Action::Quit) => {
                self.stdout.queue(cursor::SetCursorStyle::SteadyBlock)?;
                self.tx.send(Action::Quit)?;
            }
            KeyAction::Simple(Action::EnterMode(Mode::Insert)) => {
                self.tx.send(Action::EnterMode(Mode::Insert))?;
                self.stdout.queue(cursor::SetCursorStyle::SteadyBar)?;
            }
            KeyAction::Simple(Action::Hover) => {
                self.tx.send(Action::Hover)?;
            }
            KeyAction::Simple(Action::EnterMode(Mode::Normal)) => {
                self.maybe_leave_command_mode()?;
                self.tx.send(Action::EnterMode(Mode::Normal))?;
                self.stdout.queue(cursor::SetCursorStyle::SteadyBlock)?;
            }
            KeyAction::Simple(Action::EnterMode(Mode::Command)) => {
                self.tx.send(Action::EnterMode(Mode::Command))?;
                self.enter_command_mode()?;
            }
            KeyAction::Simple(Action::InsertCommand(c)) => {
                self.command.push(*c);
                self.commandline
                    .set_text(0, 0, &self.command, &self.theme.style);
                self.stdout.queue(cursor::MoveRight(1))?;
            }
            KeyAction::Simple(Action::ExecuteCommand) => {
                self.try_execute_command(mode)?;
            }
            KeyAction::Simple(Action::DeletePreviousChar) => match self.command.is_empty() {
                true => active_window.handle_action(action)?,
                false => self.delete_command_char()?,
            },
            KeyAction::Simple(_) => active_window.handle_action(action)?,
            KeyAction::Multiple(actions) => {
                for action in actions {
                    tracing::debug!("executing multiple: {action:?}");
                    self.handle_action(&KeyAction::Simple(action.clone()), mode)?;
                }
            }
            _ => (),
        };
        self.stdout
            .queue(cursor::SavePosition)?
            .queue(cursor::Hide)?;
        self.draw_statusline(&mut statusline, mode);
        self.draw_commandline(&mut commandline);
        self.render_statusline(statusline.diff(&last_statusline))?;
        self.render_commandline(commandline.diff(&last_commandline))?;
        self.statusline = statusline;
        self.commandline = commandline;
        self.stdout
            .queue(cursor::RestorePosition)?
            .queue(cursor::Show)?
            .flush()?;

        Ok(())
    }

    fn delete_command_char(&mut self) -> anyhow::Result<()> {
        match self.command.len() {
            1 => {
                self.tx.send(Action::EnterMode(Mode::Normal))?;
                self.maybe_leave_command_mode()?;
            }
            _ => {
                self.command.pop();
                tracing::debug!("command is {}", self.command);
                self.commandline
                    .set_text(0, 0, &self.command, &self.theme.style);
                self.stdout.queue(cursor::MoveLeft(1))?;
            }
        }
        Ok(())
    }

    fn try_execute_command(&mut self, mode: &Mode) -> anyhow::Result<()> {
        if let Some(action) = self.map_command_to_action(&self.command) {
            self.handle_action(&action, mode)?;
        }
        Ok(())
    }

    fn map_command_to_action(&self, command: &str) -> Option<KeyAction> {
        match command {
            ":q" => Some(KeyAction::Simple(Action::Quit)),
            ":w" => Some(KeyAction::Simple(Action::SaveBuffer)),
            ":wq" => Some(KeyAction::Multiple(vec![Action::SaveBuffer, Action::Quit])),
            _ => None,
        }
    }

    fn enter_command_mode(&mut self) -> anyhow::Result<()> {
        self.command.push(':');
        self.commandline
            .set_text(0, 0, &self.command, &self.theme.style);
        self.stdout
            .queue(cursor::SetCursorStyle::SteadyBar)?
            .queue(cursor::MoveTo(1, self.size.height as u16 - 1))?;
        Ok(())
    }

    fn maybe_leave_command_mode(&mut self) -> anyhow::Result<()> {
        if self.command.is_empty() {
            return Ok(());
        }
        let active_window = self.windows.get_mut(&self.active_window).unwrap();
        let cursor = &active_window.get_active_pane().cursor;
        let col = self.config.gutter_width + cursor.col;
        self.command = String::new();
        self.commandline.clear();
        self.stdout
            .queue(cursor::SetCursorStyle::SteadyBlock)?
            .queue(cursor::MoveTo(col as u16, cursor.row as u16))?
            .flush()?;
        Ok(())
    }

    pub fn get_active_window(&self) -> &Window {
        self.windows.get(&self.active_window).unwrap()
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.clear_screen()?;
        self.stdout.queue(terminal::LeaveAlternateScreen)?.flush()?;
        terminal::disable_raw_mode()?;

        Ok(())
    }

    pub fn initialize(&mut self, mode: &Mode) -> anyhow::Result<()> {
        terminal::enable_raw_mode()?;
        self.stdout.queue(terminal::EnterAlternateScreen)?;

        let last_statusline = self.statusline.clone();
        let last_commandline = self.commandline.clone();
        let mut statusline = Viewport::new(self.size.width, 1);
        let mut commandline = Viewport::new(self.size.width, 1);
        self.clear_screen()?;
        self.draw_statusline(&mut statusline, mode);
        self.draw_commandline(&mut commandline);
        self.render_statusline(statusline.diff(&last_statusline))?;
        self.render_commandline(commandline.diff(&last_commandline))?;

        self.windows
            .get_mut(&self.active_window)
            .unwrap()
            .initialize()?;
        self.statusline = statusline;
        self.commandline = commandline;
        self.stdout.flush()?;

        Ok(())
    }

    fn render_statusline(&mut self, changes: Vec<Change>) -> Result<()> {
        for change in changes {
            self.stdout.queue(cursor::MoveTo(
                change.col as u16,
                self.size.height as u16 - 2,
            ))?;

            if let Some(bg) = change.cell.style.bg {
                self.stdout.queue(style::SetBackgroundColor(bg))?;
            } else {
                self.stdout
                    .queue(style::SetBackgroundColor(self.theme.style.bg.unwrap()))?;
            }
            if let Some(fg) = change.cell.style.fg {
                self.stdout.queue(style::SetForegroundColor(fg))?;
            } else {
                self.stdout
                    .queue(style::SetForegroundColor(self.theme.style.fg.unwrap()))?;
            }

            self.stdout.queue(Print(change.cell.c))?;
        }
        Ok(())
    }

    fn clear_screen(&mut self) -> Result<()> {
        self.stdout
            .queue(cursor::MoveTo(0, 0))?
            .queue(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    fn draw_statusline(&mut self, viewport: &mut Viewport, mode: &Mode) {
        let active_pane = self.get_active_window().get_active_pane();
        let cursor_position = active_pane.get_cursor_readable_position();
        let Position { col, row } = cursor_position;
        let lines = active_pane.get_buffer().borrow().marker.len();

        let cursor = format!("{}:{} ", row, col);
        let percentage = match row {
            1 => "TOP ".into(),
            _ if row == lines => "BOT ".into(),
            _ => format!("{}% ", (row as f64 / lines as f64 * 100.0) as usize),
        };

        let file_name = active_pane.get_buffer().borrow().file_name.clone();
        let file_name = file_name.split('/').rev().nth(0).unwrap();
        let file_name = format!(" {}", file_name);

        let mode = format!(" {}", mode);

        let padding = " ".repeat(
            self.size.width - mode.len() - file_name.len() - cursor.len() - percentage.len(),
        );
        viewport.set_text(0, 0, &mode, &self.theme.statusline.inner);
        viewport.set_text(mode.len(), 0, &file_name, &self.theme.statusline.inner);
        viewport.set_text(
            mode.len() + file_name.len(),
            0,
            &padding,
            &self.theme.statusline.inner,
        );

        viewport.set_text(
            self.size.width - 1 - cursor.len(),
            0,
            &cursor,
            &self.theme.statusline.inner,
        );

        viewport.set_text(
            self.size.width - cursor.len(),
            0,
            &cursor,
            &self.theme.statusline.inner,
        );

        viewport.set_text(
            self.size.width - cursor.len() - percentage.len(),
            0,
            &percentage,
            &self.theme.statusline.inner,
        );
    }

    fn draw_commandline(&self, viewport: &mut Viewport) {
        let command = &self.command;
        let fill = " ".repeat(self.size.width - command.len());
        let content = format!("{}{}", command, fill);
        viewport.set_text(0, 0, &content, &self.theme.style);
    }

    fn render_commandline(&mut self, changes: Vec<Change>) -> anyhow::Result<()> {
        for change in changes {
            self.stdout.queue(cursor::MoveTo(
                change.col as u16,
                self.size.height as u16 - 1,
            ))?;

            let Style { fg, bg, .. } = change.cell.style;
            let bg = bg.expect("commandline should always have a bg");
            let fg = fg.expect("commandline should always have a fg");

            self.stdout
                .queue(style::SetBackgroundColor(bg))?
                .queue(style::SetForegroundColor(fg))?
                .queue(Print(change.cell.c))?;
        }
        Ok(())
    }

    pub fn handle_lsp_message(&mut self, message: (IncomingMessage, Option<String>)) {
        let active_window = self.windows.get_mut(&self.active_window).unwrap();
        active_window.handle_lsp_message(message);
    }
}
