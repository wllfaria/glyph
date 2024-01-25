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
    stdout: Stdout,
}

impl Pane {
    pub fn new(id: u16, buffer: Arc<Mutex<Buffer>>) -> Self {
        Self {
            id,
            row: 0,
            col: 0,
            height: 0,
            width: 0,
            buffer,
            cursor: CursorPosition { x: 5, y: 0 },
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
            let start = match i {
                r if r >= 9 => 2,
                _ => 3,
            };
            self.stdout
                .queue(cursor::MoveTo(start, i as u16))?
                .queue(Print(format!("{}", i + 1).with(Color::DarkGrey)))?
                .queue(cursor::MoveTo(5, i as u16))?
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
                if self.cursor.x > 5 {
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
                .queue(cursor::MoveTo(3, self.row + row))?
                .queue(Print("~".with(Color::DarkGrey)))?;
        }
        Ok(())
    }
}
