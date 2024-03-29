use crate::config::Config;
use crate::pane::Viewport;
use crate::theme::Theme;

use crate::pane::gutter::Gutter;

#[derive(Debug, Clone)]
pub struct AbsoluteLineGutter {
    config: Config,
    theme: Theme,
}

impl AbsoluteLineGutter {
    pub fn new(config: Config, theme: Theme) -> Self {
        Self { config, theme }
    }
}

impl Gutter for AbsoluteLineGutter {
    fn draw(&self, viewport: &mut Viewport, total_lines: usize, _: usize, scroll: usize) {
        let total_lines = usize::min(viewport.height, total_lines);
        let mut scroll = scroll;
        let style = &self.theme.gutter;

        for y in 0..total_lines {
            scroll += 1;
            let mut line = scroll.to_string();
            line = " ".repeat(self.config.gutter_width - 1 - line.len()) + &line;
            line.push(' ');

            for (x, c) in line.chars().enumerate() {
                viewport.set_cell(x, y, c, style);
            }
        }

        if total_lines < viewport.height {
            for y in total_lines..viewport.height {
                let mut line = " ".repeat(self.config.gutter_width - 2);
                line.push(self.config.empty_line_char);
                line.push(' ');

                for (x, c) in line.chars().enumerate() {
                    viewport.set_cell(x, y, c, style);
                }
            }
        }
    }
}
