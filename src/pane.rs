use std::{
    cell::RefCell,
    io::{stdout, Result, Stdout, Write},
    rc::Rc,
    sync::{Arc, Mutex},
};

use crossterm::{
    cursor,
    style::{Color, Print, Stylize},
    QueueableCommand,
};

#[derive(Debug)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

const NO_BUFFER_ATTACHED: &str = "No buffer attached to pane";

use crate::{buffer::Buffer, commands::Directions, state::State};

#[derive(Debug)]
pub struct Pane {
    pub id: u16,
    buffer: Option<Arc<Mutex<Buffer>>>,
    row: u16,
    pub col: u16,
    pub height: u16,
    pub width: u16,
    pub cursor: Position,
    pub cursor_left_limit: u16,
    col_render_offset: u16,
    stdout: Stdout,
    content_pane: Position,
    state: Rc<RefCell<State>>,
}

impl Pane {
    pub fn new(id: u16, state: Rc<RefCell<State>>) -> Self {
        Self {
            id,
            row: 0,
            col: 0,
            height: 0,
            width: 0,
            buffer: None,
            cursor: Position { x: 0, y: 0 },
            cursor_left_limit: 0,
            col_render_offset: 0,
            stdout: stdout(),
            content_pane: Position { x: 0, y: 0 },
            state,
        }
    }

    pub fn attach_buffer(&mut self, buffer: Arc<Mutex<Buffer>>) {
        let col_render_offset = buffer
            .lock()
            .expect(NO_BUFFER_ATTACHED)
            .lines
            .len()
            .to_string()
            .len() as u16
            + 1;
        let cursor_left_limit = col_render_offset + 2;
        self.cursor.x = cursor_left_limit;
        self.cursor_left_limit = cursor_left_limit;
        self.col_render_offset = col_render_offset;
        self.buffer = Some(buffer);
    }

    pub fn set_pane_position(&mut self, row: u16, col: u16, height: u16, width: u16) {
        self.row = row;
        self.col = col;
        self.height = height;
        self.width = width;
    }

    pub fn render(&mut self) -> Result<()> {
        let total_lines = self.render_lines()?;
        self.render_empty_lines(total_lines)?;
        let buffer = self
            .buffer
            .as_ref()
            .expect(NO_BUFFER_ATTACHED)
            .lock()
            .unwrap();
        let line_len = buffer.get_line_len(self.cursor.y as usize);
        let col_limit = line_len as u16 + self.cursor_left_limit;
        let cursor_col = match self.cursor.x {
            x if x > col_limit => col_limit,
            _ => self.cursor.x,
        };
        self.stdout
            .queue(cursor::MoveTo(cursor_col, self.cursor.y))?;
        Ok(())
    }

    pub fn set_cursor(&mut self, position: Position) {
        self.cursor.x = position.x + self.cursor_left_limit;
        self.cursor.y = position.y;
    }

    // TODO: I have to refactor this logic, probably make the pane have an cursor
    //       actual position and a render position.
    //       Or maybe make the line gap ignored in the calculation of the cursor
    pub fn move_cursor(&mut self, direction: &Directions) {
        let buffer = self
            .buffer
            .as_ref()
            .expect(NO_BUFFER_ATTACHED)
            .lock()
            .unwrap();
        let line_len = buffer.get_line_len(self.cursor.y as usize);
        let col_limit = line_len as u16 + self.cursor_left_limit;
        let line_above_len = match self.cursor.y {
            0 => 0,
            _ => buffer.get_line_len(self.cursor.y as usize - 1) as u16,
        };
        std::mem::drop(buffer);
        match direction {
            Directions::Up => {
                self.cursor.y = self.cursor.y.saturating_sub(1);
            }
            Directions::Down => {
                if self.cursor.y < self.height - 1 {
                    self.set_cursor(Position {
                        x: self.cursor.x - self.cursor_left_limit,
                        y: self.cursor.y + 1,
                    });
                }
            }
            Directions::Left => match self.cursor.x {
                x if x == self.cursor_left_limit && self.cursor.y > 0 => {
                    self.set_cursor(Position {
                        x: line_above_len + self.cursor_left_limit,
                        y: self.cursor.y - 1,
                    });
                }
                _ if line_len == 0 => {
                    self.set_cursor(Position {
                        x: self.cursor_left_limit,
                        y: self.cursor.y,
                    });
                }
                x if x > col_limit => {
                    self.set_cursor(Position {
                        x: col_limit - 1 - self.cursor_left_limit,
                        y: self.cursor.y,
                    });
                }
                x if x > self.cursor_left_limit => self.set_cursor(Position {
                    x: x.saturating_sub(1) - self.cursor_left_limit,
                    y: self.cursor.y,
                }),
                _ => (),
            },
            Directions::Right => match self.cursor.x {
                x if x == col_limit && self.cursor.y < self.height - 1 => {
                    self.cursor.y += 1;
                    self.cursor.x = self.cursor_left_limit;
                }
                x if x > col_limit => self.cursor.x = col_limit,
                x if x < self.width && x < col_limit => self.cursor.x += 1,
                _ => (),
            },
            Directions::LineStart => {
                self.cursor.x = self.cursor_left_limit;
            }
        }
    }

    fn render_lines(&mut self) -> Result<u16> {
        let buffer_lock = self
            .buffer
            .as_ref()
            .expect(NO_BUFFER_ATTACHED)
            .lock()
            .unwrap();
        let total_lines = usize::min(self.height as usize, buffer_lock.lines.len());

        for i in 0..total_lines {
            let readable_line = i + 1_usize;
            let line_len = readable_line.to_string().len() as u16 - 1;
            let line_display = format!("{}", readable_line).with(Color::DarkGrey);
            let line_print_col = match self.cursor.y as usize {
                y if y == i => self.col_render_offset - 2,
                _ => self.col_render_offset - line_len,
            };
            self.stdout
                .queue(cursor::MoveTo(line_print_col, i as u16))?
                .queue(Print(line_display))?
                .queue(cursor::MoveTo(self.col_render_offset + 2, i as u16))?
                .queue(Print(buffer_lock.lines.get(i).unwrap()))?;
        }

        Ok(total_lines as u16)
    }

    fn render_empty_lines(&mut self, start_row: u16) -> Result<()> {
        for row in start_row..self.height {
            self.stdout
                .queue(cursor::MoveTo(self.col_render_offset, self.row + row))?
                .queue(Print("~".with(Color::DarkGrey)))?;
        }
        Ok(())
    }
}
