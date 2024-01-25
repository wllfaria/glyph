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
    pub x: u16,
    pub y: u16,
}

use crate::{buffer::Buffer, commands::Directions};

#[derive(Debug)]
pub struct Pane {
    pub id: u16,
    pub buffer: Arc<Mutex<Buffer>>,
    pub row: u16,
    pub col: u16,
    pub height: u16,
    pub width: u16,
    pub cursor: Position,
    pub cursor_left_limit: u16,
    col_render_offset: u16,
    stdout: Stdout,
    pub content_pane: Position,
}

impl Pane {
    pub fn new(id: u16, buffer: Arc<Mutex<Buffer>>) -> Self {
        let col_render_offset = buffer.lock().unwrap().lines.len().to_string().len() as u16 + 1;
        let cursor_left_limit = col_render_offset + 2;
        Self {
            id,
            row: 0,
            col: 0,
            height: 0,
            width: 0,
            buffer,
            cursor: Position {
                x: col_render_offset + 2,
                y: 0,
            },
            cursor_left_limit,
            col_render_offset,
            stdout: stdout(),
            content_pane: Position { x: 0, y: 0 },
        }
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
        let buffer = self.buffer.lock().unwrap();
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

    pub fn move_cursor(&mut self, direction: &Directions) {
        match direction {
            Directions::Up => {
                self.cursor.y = self.cursor.y.saturating_sub(1);
            }
            Directions::Down => {
                if self.cursor.y < self.height - 1 {
                    self.cursor.y += 1;
                }
            }
            Directions::Left => {
                let buffer = self.buffer.lock().unwrap();
                let line_len = buffer.get_line_len(self.cursor.y as usize);
                let col_limit = line_len as u16 + self.cursor_left_limit;

                match self.cursor.x {
                    x if x > col_limit => self.cursor.x = col_limit - 1,
                    x if x > self.cursor_left_limit => self.cursor.x -= 1,
                    _ => (),
                }
            }
            Directions::Right => {
                let buffer = self.buffer.lock().unwrap();
                let line_len = buffer.get_line_len(self.cursor.y as usize);
                let col_limit = line_len as u16 + self.cursor_left_limit;

                match self.cursor.x {
                    x if x > col_limit => self.cursor.x = col_limit,
                    x if x < self.width && x < col_limit => self.cursor.x += 1,
                    _ => (),
                }
            }
        }
    }

    pub fn insert_line(&mut self, direction: &Directions) {
        let mut buffer_lock = self.buffer.lock().unwrap();
        match direction {
            Directions::Up => {
                buffer_lock.new_line(self.cursor.y as usize);
            }
            Directions::Down => {
                buffer_lock.new_line(self.cursor.y as usize + 1);
                self.cursor.y += 1;
            }
            _ => (),
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let mut buffer_lock = self.buffer.lock().unwrap();
        buffer_lock.insert_char(
            self.cursor.y as usize,
            self.cursor.x as usize - self.cursor_left_limit as usize,
            c,
        );
        self.cursor.x += 1;
    }

    pub fn delete_char(&mut self) {
        let mut buffer_lock = self.buffer.lock().unwrap();
        let cursor_col = self.cursor.x as usize - self.cursor_left_limit as usize;

        if cursor_col == 0 && self.cursor.y == 0 {
            return;
        }

        if cursor_col == 0 && self.cursor.y > 0 {
            let line_len = buffer_lock.get_line_len(self.cursor.y as usize - 1);
            buffer_lock.append_line(self.cursor.y as usize - 1);
            self.cursor.y -= 1;
            self.cursor.x = line_len as u16 + self.cursor_left_limit;
            return;
        }

        buffer_lock.delete_char(self.cursor.y as usize, cursor_col - 1);
        self.cursor.x -= 1;
    }

    fn render_lines(&mut self) -> Result<u16> {
        let buffer_lock = self.buffer.lock().unwrap();
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
