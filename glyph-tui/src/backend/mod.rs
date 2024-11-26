mod crossterm;

use std::io;

pub use crossterm::CrosstermBackend;

use crate::graphics::Color;

#[derive(Debug)]
pub struct Cell {
    pub symbol: String,
    pub fg: Color,
    pub bg: Color,
}

#[derive(Debug)]
pub struct Drawable<'a> {
    pub x: u16,
    pub y: u16,
    pub cell: &'a Cell,
}

pub trait Backend {
    fn setup(&mut self) -> Result<(), io::Error>;
    fn restore(&mut self) -> Result<(), io::Error>;
    fn draw<'a, I, T>(&mut self, content: I)
    where
        I: Iterator<Item = T>,
        T: Into<Drawable<'a>>;
    fn hide_cursor(&mut self) -> Result<(), io::Error>;
    fn show_cursor(&mut self) -> Result<(), io::Error>;
    fn set_cursor(&mut self, x: u16, y: u16) -> Result<(), io::Error>;
    fn flush(&mut self) -> Result<(), io::Error>;
}

impl<'a> Drawable<'a> {
    pub fn new(x: u16, y: u16, cell: &'a Cell) -> Drawable<'a> {
        Drawable { x, y, cell }
    }
}

impl<'a> From<(u16, u16, &'a Cell)> for Drawable<'a> {
    fn from((x, y, cell): (u16, u16, &'a Cell)) -> Drawable<'a> {
        Drawable::new(x, y, cell)
    }
}
