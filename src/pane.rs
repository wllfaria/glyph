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
pub struct CursorPosition {
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
    pub cursor: CursorPosition,
    cursor_left_limit: u16,
    col_render_offset: u16,
    stdout: Stdout,
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
            cursor: CursorPosition {
                x: col_render_offset + 2,
                y: 0,
            },
            cursor_left_limit,
            col_render_offset,
            stdout: stdout(),
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
        self.stdout
            .queue(cursor::MoveTo(self.cursor.x, self.cursor.y))?;
        Ok(())
    }

    fn render_lines(&mut self) -> Result<u16> {
        let buffer_lock = self.buffer.lock().unwrap();
        let total_lines = usize::min(self.height as usize, buffer_lock.lines.len());
        for i in 0..total_lines {
            let normalized_line = i + 1 as usize;
            let line_col = normalized_line.to_string().len() as u16 - 1;
            let line_number = format!("{}", normalized_line).with(Color::DarkGrey);
            if i == self.cursor.y as usize {
                self.stdout
                    .queue(cursor::MoveTo(self.col_render_offset - 2, i as u16))?
                    .queue(Print(line_number.with(Color::DarkGreen)))?;
            } else {
                self.stdout
                    .queue(cursor::MoveTo(self.col_render_offset - line_col, i as u16))?
                    .queue(Print(line_number))?;
            }
            self.stdout
                .queue(cursor::MoveTo(self.col_render_offset + 2, i as u16))?
                .queue(Print(buffer_lock.lines.get(i).unwrap()))?;
        }
        Ok(total_lines as u16)
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
                if self.cursor.x > self.cursor_left_limit {
                    self.cursor.x -= 1;
                }
            }
            Directions::Right => {
                if self.cursor.x < self.width {
                    self.cursor.x += 1;
                }
            }
        }
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
