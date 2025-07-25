use std::io::{Write, stdout};
use std::sync::Arc;

use crossterm::style::{Attribute, Color, Print, SetAttribute};
use crossterm::{cursor, queue};
use glyph_core::config::{Config, StatuslineMode};
use glyph_core::geometry::{Point, Rect, Size};
use glyph_core::renderer::error::{RendererError, Result};
use glyph_core::renderer::{RenderContext, Renderer};
use glyph_core::status_provider::StatuslineContext;
use glyph_core::view_manager::{LayoutTreeNode, LeafView};

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

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fg(self, fg: Color) -> Self {
        Self {
            fg,
            bg: self.bg,
            bold: self.bold,
            italic: self.italic,
            underline: self.underline,
        }
    }

    pub fn with_bg(self, bg: Color) -> Self {
        Self {
            fg: self.fg,
            bg,
            bold: self.bold,
            italic: self.italic,
            underline: self.underline,
        }
    }

    pub fn bold(self) -> Self {
        Self {
            fg: self.fg,
            bg: self.bg,
            bold: true,
            italic: self.italic,
            underline: self.underline,
        }
    }

    pub fn italic(self) -> Self {
        Self {
            fg: self.fg,
            bg: self.bg,
            bold: self.bold,
            italic: true,
            underline: self.underline,
        }
    }

    pub fn underline(self) -> Self {
        Self {
            fg: self.fg,
            bg: self.bg,
            bold: self.bold,
            italic: self.italic,
            underline: true,
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

#[derive(Debug)]
pub struct Change {
    cell: Cell,
    position: Point<u16>,
}

impl Change {
    pub fn new(cell: Cell, position: Point<u16>) -> Self {
        Self { cell, position }
    }
}

#[derive(Debug)]
pub struct ChangeSet {
    pub changes: Vec<Change>,
}

#[derive(Debug, Default)]
pub struct CellBuffer(Vec<Cell>);

impl CellBuffer {
    pub fn diff(&self, other: &Self, size: Size) -> ChangeSet {
        let mut changes = vec![];

        for (i, c) in self.0.iter().enumerate() {
            let x = i % size.width as usize;
            let y = i / size.width as usize;

            if c != &other.0[i] {
                changes.push(Change::new(*c, Point::new(x as u16, y as u16)));
            }
        }

        ChangeSet { changes }
    }

    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell, size: Size) {
        let index = y as usize * size.width as usize + x as usize;
        self.0[index] = cell;
    }
}

#[derive(Debug)]
pub struct CrosstermRenderer {
    size: Size,
    config: Arc<Config>,
    buffers: [CellBuffer; 2],
}

impl CrosstermRenderer {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let mut renderer = Self {
            config,
            size: Size::default(),
            buffers: [CellBuffer::default(), CellBuffer::default()],
        };

        renderer.resize(renderer.get_size()?)?;

        Ok(renderer)
    }

    fn swap_buffers(&mut self) {
        self.buffers.swap(0, 1);
    }

    fn render_layout_node(&mut self, ctx: &RenderContext<'_>, node: &LayoutTreeNode, rect: Rect) {
        match node {
            LayoutTreeNode::Leaf(leaf) => self.render_leaf_view(ctx, leaf, rect),
            LayoutTreeNode::Split(split) => split
                .children
                .iter()
                .for_each(|c| self.render_layout_node(ctx, c, split.rect)),
        }
    }

    fn render_leaf_view(&mut self, ctx: &RenderContext<'_>, leaf: &LeafView, rect: Rect) {
        let cell_buffer = &mut self.buffers[0];
        let visible_views = ctx.views.get_visible();
        let view = visible_views.iter().find(|v| v.id == leaf.view_id).unwrap();
        let buffer = ctx.buffers.iter().find(|b| b.id == view.buffer_id).unwrap();
        let content = buffer.content();

        for y in 0..leaf.usable_rect.height as usize {
            let line = content.get_line(y + view.scroll_offset.y);

            for x in 0..leaf.usable_rect.width as usize {
                let char = line
                    .and_then(|line| line.get_char(x + view.scroll_offset.x))
                    .unwrap_or(' ');

                let char = match char {
                    '\n' => ' ',
                    '\t' => ' ',
                    '\r' => ' ',
                    _ => char,
                };

                let cell = Cell::new(char, Style::default());
                let screen_x = rect.x + leaf.usable_rect.x + x as u16;
                let screen_y = rect.y + leaf.usable_rect.y + y as u16;
                cell_buffer.set_cell(screen_x, screen_y, cell, self.size);
            }
        }

        self.maybe_render_view_statusline(ctx, leaf);
    }

    fn maybe_render_view_statusline(&self, ctx: &RenderContext<'_>, leaf: &LeafView) {
        let view = ctx.views.get_active_view();
        // TODO: maybe this search is not very good
        let buffer_info = ctx.buffers.iter().find(|b| b.id == view.buffer_id).unwrap();
        let cursor = view.cursors.first().unwrap();
        let cursor_position = Point::new(cursor.x, cursor.y);

        let statusline_str = ctx
            .statusline_provider
            .render_statusline(&StatuslineContext {
                buffer_info,
                cursor_position,
                current_mode: ctx.mode,
                width: leaf.rect.width as usize,
            });

        let y = leaf.rect.bottom();

        _ = crossterm::queue!(
            stdout(),
            crossterm::cursor::MoveTo(0, y),
            Print(statusline_str),
        );
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

    fn position_cursor(&self, ctx: &mut RenderContext<'_>) {
        let view = ctx.views.get_active_view();
        let cursor = view.cursors.first().unwrap();
        let cursor_x = cursor.x - view.scroll_offset.x;
        let cursor_y = cursor.y - view.scroll_offset.y;
        _ = queue!(stdout(), cursor::MoveTo(cursor_x as u16, cursor_y as u16));
    }
}

impl Renderer for CrosstermRenderer {
    fn render(&mut self, ctx: &mut RenderContext<'_>) -> Result<()> {
        _ = queue!(stdout(), cursor::Hide);

        let editor_rect = Rect::with_size(0, 0, self.size);

        self.render_layout_node(ctx, ctx.layout, editor_rect);

        let changes = self.buffers[0].diff(&self.buffers[1], self.size);
        for change in changes.changes {
            let x = change.position.x;
            let y = change.position.y;
            self.queue_change(x, y, change)?;
        }

        self.position_cursor(ctx);

        _ = queue!(stdout(), cursor::Show);

        if let Err(e) = stdout().flush() {
            return Err(RendererError::FailedToFlushRenderer(e));
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

        let statusline_offset = match self.config.statusline.mode {
            StatuslineMode::Global => 1,
            StatuslineMode::Local => 0,
        };

        Ok(Size::new(width, height - statusline_offset))
    }

    fn resize(&mut self, size: Size) -> Result<()> {
        self.size = size;
        let buffer = vec![Cell::default(); size.width as usize * size.height as usize];
        self.buffers = [CellBuffer(buffer.clone()), CellBuffer(buffer)];
        Ok(())
    }
}