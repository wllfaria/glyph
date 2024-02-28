use crate::pane::Position;
use crate::theme::Theme;
use crate::viewport::Viewport;
use crate::{pane::Rect, viewport::Cell};

use super::{Renderable, TuiView};

pub struct HoverPopup<'a> {
    area: Rect,
    view: Box<dyn Renderable + 'a>,
    viewport: Viewport,
    content: &'a str,
    theme: &'a Theme,
}

impl<'a> HoverPopup<'a> {
    pub fn new(area: Rect, theme: &'a Theme, content: &'a str) -> Self {
        Self {
            area: area.clone(),
            viewport: Viewport::new(area.width, area.height),
            theme,
            content,
            view: Box::new(TuiView::new(area, theme, 0)),
        }
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        let cells = self.content_to_vec_cells();
        self.view
            .draw(&mut self.viewport, &cells, &Position::default());
        self.view
            .render_diff(&Viewport::new(0, 0), &self.viewport)?;
        Ok(())
    }

    fn content_to_vec_cells(&self) -> Vec<Cell> {
        let style = &self.theme.style;
        self.content
            .chars()
            .map(|c| Cell {
                c,
                style: style.clone(),
            })
            .collect()
    }
}
