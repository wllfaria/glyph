use std::io::stdout;

use crossterm::style::{Attribute, Color, Print, SetAttribute};
use glyph_core::geometry::{Point, Rect, Size};
use glyph_core::renderer::error::{RendererError, Result};
use glyph_core::renderer::{RenderContext, Renderer};
use glyph_core::view_manager::LayoutTreeNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
    pub char: char,
    pub style: Style,
}

impl Cell {
    pub fn new(char: char, style: Style) -> Self {
        Self { char, style }
    }
}

pub struct Change {
    cell: Cell,
    position: Point,
}

pub struct ChangeSet {
    pub changes: Vec<Change>,
}

#[derive(Debug, Default)]
pub struct CellBuffer(Vec<Cell>);

impl CellBuffer {
    pub fn diff(&self, other: &Self) -> ChangeSet {
        let mut changes = vec![];

        for (i, c) in self.0.iter().enumerate() {
            let x = i % self.0.len();
            let y = i / self.0.len();

            if c != &other.0[i] {
                changes.push(Change {
                    cell: *c,
                    position: Point::new(x as u16, y as u16),
                });
            }
        }

        ChangeSet { changes }
    }

    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell) {
        let len = self.0.len();
        self.0[y as usize * len + x as usize] = cell;
    }
}

#[derive(Debug)]
pub struct CrosstermRenderer {
    size: Size,
    buffers: [CellBuffer; 2],
}

impl CrosstermRenderer {
    pub fn new() -> Result<Self> {
        let mut renderer = Self {
            buffers: [CellBuffer::default(), CellBuffer::default()],
            size: Size::default(),
        };

        renderer.resize(renderer.get_size()?)?;

        Ok(renderer)
    }

    fn swap_buffers(&mut self) {
        self.buffers.swap(0, 1);
    }

    fn queue_change(&mut self, x: u16, y: u16, change: Change) -> Result<()> {
        let mut stdout = stdout();

        if change.cell.style.bold {
            _ = crossterm::queue!(stdout, SetAttribute(Attribute::Bold));
        }

        if change.cell.style.italic {
            _ = crossterm::queue!(stdout, SetAttribute(Attribute::Italic));
        }

        if change.cell.style.underline {
            _ = crossterm::queue!(stdout, SetAttribute(Attribute::Underlined));
        }

        _ = crossterm::queue!(
            stdout,
            crossterm::cursor::MoveTo(x, y),
            crossterm::style::SetForegroundColor(change.cell.style.fg),
            crossterm::style::SetBackgroundColor(change.cell.style.bg),
            Print(change.cell.char)
        );

        Ok(())
    }
}

pub struct LayoutWalker<'a> {
    foo: &'a mut u8,
}

impl Renderer for CrosstermRenderer {
    fn render(&mut self, ctx: &mut RenderContext<'_>) -> Result<()> {
        let cell_buffer = &mut self.buffers[0];
        let mut editor_rect = Rect::with_size(0, 0, self.size);
        editor_rect.cut_bottom(1);

        match ctx.layout {
            LayoutTreeNode::Leaf(leaf) => {
                let view = ctx.views.iter().find(|v| v.id == leaf.view_id).unwrap();
                let buffer = ctx.buffers.iter().find(|b| b.id == view.buffer_id).unwrap();

                for (y, line) in buffer
                    .content()
                    .lines()
                    .skip(view.scroll_offset.y as usize)
                    .take(leaf.size.height as usize - 1)
                    .enumerate()
                {
                    for (x, char) in line
                        .chars()
                        .skip(view.scroll_offset.x as usize)
                        .take(leaf.size.width as usize - 1)
                        .enumerate()
                    {
                        let cell = Cell::new(char, Style::default());
                        cell_buffer.set_cell(x as u16, y as u16, cell);
                    }
                }
            }
            _ => todo!(),
        }

        println!("{cell_buffer:?}");

        let changes = self.buffers[0].diff(&self.buffers[1]);
        for change in changes.changes {
            let x = change.position.x;
            let y = change.position.y;
            self.queue_change(x, y, change)?;
        }

        self.swap_buffers();

        Ok(())
    }

    fn setup(&self) -> Result<()> {
        if crossterm::terminal::enable_raw_mode().is_err() {
            return Err(RendererError::FailedToSetupRenderer);
        }

        if crossterm::execute!(stdout(), crossterm::terminal::EnterAlternateScreen).is_err() {
            _ = crossterm::terminal::disable_raw_mode();
            return Err(RendererError::FailedToSetupRenderer);
        }

        Ok(())
    }

    fn shutdown(&self) -> Result<()> {
        if crossterm::execute!(stdout(), crossterm::terminal::LeaveAlternateScreen).is_err() {
            return Err(RendererError::FailedToShutdownRenderer);
        }

        if crossterm::terminal::disable_raw_mode().is_err() {
            return Err(RendererError::FailedToShutdownRenderer);
        }

        Ok(())
    }

    fn get_size(&self) -> Result<Size> {
        let (width, height) = match crossterm::terminal::size() {
            Ok(size) => size,
            Err(_) => return Err(RendererError::FailedToGetEditorSize),
        };

        Ok(Size::new(width, height))
    }

    fn resize(&mut self, size: Size) -> Result<()> {
        self.size = size;
        let buffer = vec![Cell::default(); size.width as usize * size.height as usize];
        self.buffers = [CellBuffer(buffer.clone()), CellBuffer(buffer)];
        Ok(())
    }
}
