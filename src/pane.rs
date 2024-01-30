use crossterm::{
    cursor,
    style::{Color, Print, Stylize},
    QueueableCommand,
};
use std::{
    cell::RefCell,
    io::{stdout, Result, Stdout},
    rc::Rc,
};

use crate::{
    command::{
        BufferCommands, Command, CommandBus, CommandListener, CursorCommands, EditorCommands,
        PaneCommands,
    },
    cursor::Cursor,
};

#[derive(Debug)]
pub struct Position {
    pub row: u16,
    pub col: u16,
    pub render_col: u16,
}

pub struct Pane {
    pub id: u16,
    pub pane_size: PaneSize,
    pub content_size: PaneSize,
    pub sidebar_width: u16,
    pub sidebar_gap: u16,
    cursor: Rc<RefCell<Cursor>>,
    command_bus: Rc<RefCell<CommandBus>>,
    stdout: Stdout,
}

pub struct PaneSize {
    pub row: u16,
    pub col: u16,
    pub height: u16,
    pub width: u16,
}

impl Pane {
    pub fn new(id: u16, command_bus: Rc<RefCell<CommandBus>>) -> Self {
        let cursor = Rc::new(RefCell::new(Cursor::new(command_bus.clone())));
        cursor.borrow_mut().setup(cursor.clone());

        Self {
            id,
            sidebar_width: 5,
            sidebar_gap: 1,
            cursor: cursor.clone(),
            stdout: stdout(),
            command_bus,
            pane_size: PaneSize {
                row: 10,
                col: 10,
                height: 0,
                width: 0,
            },
            content_size: PaneSize {
                row: 0,
                col: 0,
                height: 0,
                width: 0,
            },
        }
    }

    pub fn setup(&mut self, pane: Rc<RefCell<Pane>>) {
        self.command_bus
            .borrow_mut()
            .subscribe(Box::new(PaneListener { pane }));
    }

    pub fn resize_pane(&mut self, size: PaneSize) {
        self.content_size = PaneSize {
            row: size.row,
            col: size.col + self.sidebar_width + self.sidebar_gap,
            width: size.width - self.sidebar_width - self.sidebar_gap,
            height: size.height,
        };
        self.pane_size = size;
    }

    fn render_buffer(&mut self, lines: &[String]) -> Result<()> {
        for (i, line) in lines.iter().enumerate() {
            self.stdout
                .queue(cursor::MoveTo(
                    self.content_size.col,
                    self.content_size.row + i as u16,
                ))?
                .queue(Print(line))?;
        }
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        // NOTE: Order matters here. We need to render the sidebar first
        self.render_sidebar(0)?;
        self.render_cursor()?;
        Ok(())
    }

    fn render_cursor(&mut self) -> Result<()> {
        self.stdout.queue(cursor::MoveTo(
            self.content_size.col + self.cursor.borrow().col,
            self.cursor.borrow().row,
        ))?;
        Ok(())
    }

    fn render_sidebar(&mut self, _: u16) -> Result<()> {
        for row in 0..self.pane_size.height {
            self.stdout
                .queue(cursor::MoveTo(
                    self.pane_size.col + self.sidebar_width - self.sidebar_gap,
                    self.content_size.row + row,
                ))?
                .queue(Print("~".with(Color::DarkGrey)))?;
        }
        Ok(())
    }
}

pub struct PaneListener {
    pane: Rc<RefCell<Pane>>,
}

impl CommandListener for PaneListener {
    fn call(&mut self, command: &Command, _id: u16) -> Result<()> {
        match command {
            Command::Editor(EditorCommands::Start) => self.pane.borrow_mut().render()?,
            Command::Buffer(BufferCommands::Type(_)) => self.pane.borrow_mut().render()?,
            Command::Buffer(BufferCommands::Backspace) => self.pane.borrow_mut().render()?,
            Command::Buffer(BufferCommands::NewLineBelow) => self.pane.borrow_mut().render()?,
            Command::Cursor(CursorCommands::MoveUp) => self.pane.borrow_mut().render()?,
            Command::Cursor(CursorCommands::MoveDown) => self.pane.borrow_mut().render()?,
            Command::Cursor(CursorCommands::MoveLeft) => self.pane.borrow_mut().render()?,
            Command::Cursor(CursorCommands::MoveRight) => self.pane.borrow_mut().render()?,
            Command::Pane(PaneCommands::BufferUpdate(lines)) => {
                self.pane.borrow_mut().render_buffer(lines)?
            }
            _ => {}
        }
        Ok(())
    }
}
