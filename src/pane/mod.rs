use std::cell::RefCell;
use std::rc::Rc;

use crate::buffer::Buffer;
use crate::config::{Action, Config, KeyAction, LineNumbers};
use crate::cursor::Cursor;
use crate::editor::Mode;
use crate::highlight::Highlight;
use crate::lsp::IncomingMessage;
use crate::pane::gutter::Gutter;
use crate::theme::Theme;
use crate::tui::Rect;
use crate::viewport::{Cell, Viewport};

use self::gutter::absolute_line_gutter::AbsoluteLineGutter;
use self::gutter::noop_line_gutter::NoopLineDrawer;
use self::gutter::relative_line_gutter::RelativeLineDrawer;

mod gutter;

#[derive(Debug, Default, Clone)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

pub struct Pane<'a> {
    pub id: usize,
    pub cursor: Cursor,
    highlight: Highlight<'a>,
    // buffer_view: Box<dyn Scrollable + 'a>,
    pub buffer: Rc<RefCell<Buffer>>,
    /// Currently, `layers[0]` is the buffer layer and `layers[1]` is the popups layer
    config: &'a Config,
    gutter: Box<dyn Gutter>,
    pub size: Rect,
    theme: &'a Theme,
}

impl<'a> Pane<'a> {
    pub fn new(
        id: usize,
        buffer: Rc<RefCell<Buffer>>,
        theme: &'a Theme,
        config: &'a Config,
    ) -> Self {
        let gutter: Box<dyn Gutter> = match config.line_numbers {
            LineNumbers::Absolute => {
                Box::new(AbsoluteLineGutter::new(config.clone(), theme.clone()))
            }
            LineNumbers::Relative => {
                Box::new(RelativeLineDrawer::new(config.clone(), theme.clone()))
            }
            LineNumbers::RelativeNumbered => {
                Box::new(RelativeLineDrawer::new(config.clone(), theme.clone()))
            }
            LineNumbers::None => Box::new(NoopLineDrawer::new(config.clone(), theme.clone())),
        };

        let size: Rect = (1, 1).into();

        Self {
            id,
            buffer,
            highlight: Highlight::new(theme),
            cursor: Cursor::new(),
            // buffer_view: Box::new(TuiView::new(size.clone(), config.gutter_width)),
            size,
            gutter,
            config,
            theme,
        }
    }

    pub fn initialize(&mut self, mode: &Mode) -> anyhow::Result<()> {
        // self.layers[0] = Viewport::new(self.size.width, self.size.height);
        // self.layers[1] = Viewport::new(self.size.width, self.size.height);
        // self.render(mode)?;
        Ok(())
    }

    pub fn resize(&mut self, new_size: Rect, mode: &Mode) -> anyhow::Result<()> {
        self.size = new_size.clone();
        // self.buffer_view.resize(new_size, self.config.gutter_width);
        self.render(mode)?;
        Ok(())
    }

    pub fn handle_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        match action {
            KeyAction::Simple(Action::MoveToLineStart) => {
                self.handle_cursor_action(action, mode)?
            }
            KeyAction::Simple(Action::MoveToLineEnd) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::DeletePreviousChar) => {
                self.handle_buffer_action(action, mode)?
            }
            KeyAction::Simple(Action::DeleteCurrentChar) => {
                self.handle_buffer_action(action, mode)?
            }
            KeyAction::Simple(Action::NextWord) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::MoveLeft) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::MoveDown) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::MoveUp) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::MoveRight) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::MoveToTop) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::SaveBuffer) => self.handle_buffer_action(action, mode)?,
            KeyAction::Simple(Action::MoveToBottom) => self.handle_cursor_action(action, mode)?,
            KeyAction::Simple(Action::InsertLine) => self.handle_buffer_action(action, mode)?,
            KeyAction::Simple(Action::InsertLineBelow) => {
                self.handle_buffer_action(action, mode)?
            }
            KeyAction::Simple(Action::InsertLineAbove) => {
                self.handle_buffer_action(action, mode)?
            }
            KeyAction::Simple(Action::InsertChar(_)) => self.handle_buffer_action(action, mode)?,
            _ => (),
        };

        self.render(mode)?;
        Ok(())
    }

    fn render(&mut self, mode: &Mode) -> anyhow::Result<()> {
        // self.buffer_view.hide_cursor()?;
        self.render_buffer()?;
        // self.buffer_view
        // .draw_cursor(mode, &self.buffer.borrow(), &self.cursor)?;
        // self.buffer_view.show_cursor()?;
        Ok(())
    }

    fn render_buffer(&mut self) -> anyhow::Result<()> {
        // let last_buffer_layer = self.layers[0].clone();
        // let mut viewport = Viewport::new(self.size.width, self.size.height);

        // self.buffer_view.maybe_scroll(&self.cursor);
        // self.draw_sidebar(&mut viewport);
        let cells = self.get_highlight();
        // self.buffer_view.draw(&mut viewport, &cells);
        // self.buffer_view
        // .render_diff(&last_buffer_layer, &viewport, &self.theme.style)?;
        // self.layers[0] = viewport;
        Ok(())
    }

    fn get_highlight(&mut self) -> Vec<Cell> {
        let mut result: Vec<Cell> = Vec::new();
        let mut current_byte_index = 0;
        // let scroll = self.buffer_view.get_scroll();
        // let buffer = self
        //     .buffer
        //     .borrow()
        //     .content_from(scroll.row, self.size.height);
        // let colors = self.highlight.colors(&buffer);
        // let style = self.theme.style;
        //
        // for c in buffer.chars() {
        //     let cell = match colors
        //         .iter()
        //         .find(|token| current_byte_index >= token.start && current_byte_index < token.end)
        //     {
        //         Some(token) => Cell {
        //             c,
        //             style: *token.style,
        //         },
        //         None => Cell { c, style },
        //     };
        //     result.push(cell);
        //     current_byte_index += c.len_utf8();
        // }
        //
        // result
        vec![]
    }

    pub fn get_cursor_readable_position(&self) -> Position {
        self.cursor.get_readable_position()
    }

    pub fn handle_cursor_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        self.cursor
            .handle(action, &mut self.buffer.borrow_mut(), mode);
        Ok(())
    }

    fn handle_buffer_action(&mut self, action: &KeyAction, mode: &Mode) -> anyhow::Result<()> {
        let col = self.cursor.col;
        let row = self.cursor.row;
        let mark = {
            let buffer = self.buffer.borrow_mut();
            let mark = buffer.marker.get_by_cursor(self.cursor.absolute_position);
            mark.unwrap()
        };

        self.buffer
            .borrow_mut()
            .handle_action(action, self.cursor.absolute_position)?;

        self.handle_cursor_action(action, mode)?;

        if let KeyAction::Simple(Action::DeletePreviousChar) = action {
            if let (0, 1..) = (col, row) {
                self.cursor.col = mark.size.saturating_sub(1);
                self.cursor.absolute_position = mark.start + mark.size.saturating_sub(1);
            }
        };

        Ok(())
    }

    fn draw_sidebar(&mut self, viewport: &mut Viewport) {
        // let scroll = self.buffer_view.get_scroll();
        // self.gutter.draw(
        //     viewport,
        //     self.buffer.borrow().marker.len(),
        //     self.cursor.row,
        //     scroll.row,
        // );
    }

    pub fn get_buffer(&self) -> Rc<RefCell<Buffer>> {
        self.buffer.clone()
    }
}
