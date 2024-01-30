use std::{
    io::{stdout, Result, Stdout},
    sync::{Arc, Mutex},
};

use crossterm::{
    cursor,
    style::{Color, Print, Stylize},
    QueueableCommand,
};

#[derive(Debug)]
pub struct Position {
    pub row: u16,
    pub col: u16,
    pub render_col: u16,
}

impl Position {
    pub fn new(col: u16, row: u16, render_col: u16) -> Self {
        Self {
            col,
            row,
            render_col,
        }
    }
}

const NO_BUFFER_ATTACHED: &str = "No buffer attached to pane";

use crate::buffer::Buffer;

#[derive(Debug)]
pub struct Pane {
    pub id: u16,
    pub cursor: Position,
    buffer: Rc<RefCell<Buffer>>,
    pub pane_size: PaneSize,
    pub content_size: PaneSize,
    pub sidebar_width: u16,
    pub sidebar_gap: u16,
    stdout: Stdout,
}

#[derive(Debug)]
pub struct PaneSize {
    pub row: u16,
    pub col: u16,
    pub height: u16,
    pub width: u16,
}

impl Pane {
    pub fn new(id: u16, buffer: Rc<RefCell<Buffer>>) -> Self {
        Self {
            id,
            buffer,
            sidebar_width: 5,
            sidebar_gap: 1,
            stdout: stdout(),
            cursor: Position {
                col: 0,
                row: 0,
                render_col: 0,
            },
            pane_size: PaneSize {
                row: 0,
                col: 0,
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

    pub fn resize_pane(&mut self, size: PaneSize) {
        self.content_size = PaneSize {
            row: size.row,
            col: size.col + self.sidebar_width + self.sidebar_gap,
            width: size.width - self.sidebar_width - self.sidebar_gap,
            height: size.height,
        };
        self.pane_size = size;
    }

    pub fn set_cursor(&mut self, position: Position) {
        self.cursor = position;
    }

    pub fn render(&mut self) -> Result<()> {
        let total_lines = self.render_lines()?;
        self.render_empty_lines(total_lines)?;
        let column = self.content_size.col + self.cursor.render_col;
        self.stdout.queue(cursor::MoveTo(column, self.cursor.row))?;
        Ok(())
    }

    pub fn move_cursor(&mut self, direction: bool) {
        let buffer = self
            .buffer
            .as_ref()
            .expect(NO_BUFFER_ATTACHED)
            .lock()
            .unwrap();
        let line_len = buffer.get_line_len(self.cursor.row as usize) as u16;
        let line_above_len = match self.cursor.row {
            0 => 0,
            _ => buffer.get_line_len(self.cursor.row as usize - 1) as u16,
        };
        let line_below_len = match self.cursor.row {
            x if x >= self.content_size.height => 0,
            _ => buffer.get_line_len(self.cursor.row as usize + 1) as u16,
        };
        std::mem::drop(buffer);
        match direction {
            Directions::Up => match self.cursor.col {
                col if col > line_above_len && self.cursor.row == 0 => self.set_cursor(
                    Position::new(line_above_len, self.cursor.row, line_above_len),
                ),
                col if col > line_above_len => self.set_cursor(Position::new(
                    self.cursor.col,
                    self.cursor.row.saturating_sub(1),
                    line_above_len,
                )),
                _ => {
                    self.set_cursor(Position::new(
                        self.cursor.col,
                        self.cursor.row.saturating_sub(1),
                        self.cursor.col,
                    ));
                }
            },
            Directions::Down => match self.cursor.row {
                row if row >= self.content_size.height - 1 => (),
                _ => match self.cursor.col {
                    col if col > line_below_len => self.set_cursor(Position::new(
                        self.cursor.col,
                        self.cursor.row + 1,
                        line_below_len,
                    )),
                    _ => self.set_cursor(Position::new(
                        self.cursor.col,
                        self.cursor.row + 1,
                        self.cursor.col,
                    )),
                },
            },
            Directions::Left => match self.cursor.col {
                col if col == 0 && self.cursor.row == 0 => (),
                col if col == 0 && self.cursor.row > 0 => self.set_cursor(Position::new(
                    line_above_len,
                    self.cursor.row - 1,
                    line_above_len,
                )),
                col if col > line_len && line_len == 0 => self.set_cursor(Position::new(
                    line_above_len,
                    self.cursor.row - 1,
                    line_above_len,
                )),
                col if col > line_len => {
                    self.set_cursor(Position::new(line_len, self.cursor.row, line_len))
                }
                _ => self.set_cursor(Position::new(
                    self.cursor.col - 1,
                    self.cursor.row,
                    self.cursor.col - 1,
                )),
            },
            Directions::Right => match self.cursor.col {
                _ if self.content_size.height - 1 == self.cursor.row => (),
                col if col >= line_len => self.set_cursor(Position::new(0, self.cursor.row + 1, 0)),
                col if col >= line_len && self.cursor.row == self.content_size.height => {
                    self.set_cursor(Position::new(0, self.cursor.row, 0))
                }
                _ => self.set_cursor(Position::new(
                    self.cursor.col + 1,
                    self.cursor.row,
                    self.cursor.render_col + 1,
                )),
            },
            Directions::LineStart => self.set_cursor(Position::new(0, self.cursor.row, 0)),
        }
    }

    fn render_lines(&mut self) -> Result<u16> {
        let buffer_lock = self
            .buffer
            .as_ref()
            .expect(NO_BUFFER_ATTACHED)
            .lock()
            .unwrap();
        let total_lines = usize::min(self.pane_size.height as usize, buffer_lock.lines.len());

        for i in 0..total_lines {
            let readable_line = i + 1_usize;
            let line_len = readable_line.to_string().len() as u16;
            let line_display = format!("{}", readable_line).with(Color::DarkGrey);
            let line_print_col = self.pane_size.col + self.sidebar_width - line_len;
            self.stdout
                .queue(cursor::MoveTo(line_print_col, i as u16))?
                .queue(Print(line_display))?
                .queue(cursor::MoveTo(self.content_size.col, i as u16))?
                .queue(Print(buffer_lock.lines.get(i).unwrap()))?;
        }

        Ok(total_lines as u16)
    }

    fn render_empty_lines(&mut self, start_row: u16) -> Result<()> {
        for row in start_row..self.pane_size.height {
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
