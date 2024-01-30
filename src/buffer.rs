use std::{cell::RefCell, io::Result, rc::Rc};

use crate::{
    command::{BufferCommands, Command, CommandBus, CommandListener, PaneCommands},
    pane::Position,
};

pub struct Buffer {
    pub id: u16,
    lines: Vec<String>,
    command_bus: Rc<RefCell<CommandBus>>,
}

impl Buffer {
    pub fn new(id: u16, filename: Option<String>, command_bus: Rc<RefCell<CommandBus>>) -> Self {
        let lines = match filename {
            Some(filename) => {
                let lines = std::fs::read_to_string(filename).unwrap();
                lines.lines().map(|s| s.to_string()).collect()
            }
            None => Vec::new(),
        };
        Buffer {
            id,
            lines,
            command_bus,
        }
    }

    pub fn setup(&self, buffer: Rc<RefCell<Buffer>>) {
        self.command_bus
            .borrow_mut()
            .subscribe(Box::new(BufferListener { buffer }));
    }

    pub fn new_line(&mut self, current_row: usize, col: usize) -> Result<()> {
        match col {
            _ if self.lines.len() == 0 => {
                self.lines.push(String::new());
            }
            c if c < self.lines[current_row].len() => {
                self.split_line(current_row, col);
            }
            _ => {
                self.lines.insert(current_row + 1, String::new());
            }
        }
        self.notify_listeners()?;
        Ok(())
    }

    fn notify_listeners(&self) -> Result<()> {
        self.command_bus
            .borrow_mut()
            .dispatch::<PaneCommands>(Command::Pane(PaneCommands::BufferUpdate(&self.lines)))?;
        Ok(())
    }

    pub fn insert_char(&mut self, row: usize, col: usize, c: char) -> Result<()> {
        match col {
            col if col >= self.lines[row].len() => self.lines[row].push(c),
            _ => self.lines[row].insert(col, c),
        }
        self.notify_listeners()?;
        Ok(())
    }

    pub fn delete_char(&mut self, row: usize, col: usize) -> Position {
        match col {
            c if c == 0 && row == 0 => Position {
                col: 0,
                row: 0,
                render_col: 0,
            },
            c if c == 0 && row > 0 => {
                let cursor = Position {
                    col: self.get_line_len(row - 1) as u16,
                    row: row as u16 - 1,
                    render_col: self.get_line_len(row - 1) as u16,
                };
                self.append_line(row - 1);
                return cursor;
            }
            c if c >= self.lines[row].len() => {
                self.lines[row].pop();
                return Position {
                    col: self.get_line_len(row) as u16,
                    row: row as u16,
                    render_col: self.get_line_len(row) as u16,
                };
            }
            _ => {
                let left = self.lines[row][..col - 1].to_string();
                let right = self.lines[row][col..].to_string();
                self.lines[row] = left + &right;
                return Position {
                    col: col as u16 - 1,
                    row: row as u16,
                    render_col: col as u16 - 1,
                };
            }
        }
    }

    pub fn split_line(&mut self, line: usize, col: usize) {
        let first = self.lines[line][..col].to_string();
        let second = self.lines[line][col..].to_string();
        self.lines[line] = first.to_string();
        self.lines.insert(line + 1, String::new());
        self.lines[line + 1] = second.to_string();
    }

    pub fn append_line(&mut self, line: usize) {
        let next = self.lines[line + 1].to_string();
        self.lines[line].push_str(&next);
        self.lines.remove(line + 1);
    }

    pub fn get_line_len(&self, line: usize) -> usize {
        match line {
            l if l >= self.lines.len() => 0,
            _ if self.lines.len() == 0 => 0,
            _ => self.lines[line].len(),
        }
    }
}

pub struct BufferListener {
    buffer: Rc<RefCell<Buffer>>,
}

impl CommandListener for BufferListener {
    fn call(&mut self, command: &Command, _: u16) -> std::io::Result<()> {
        match command {
            _ => {}
        };
        Ok(())
    }
}
