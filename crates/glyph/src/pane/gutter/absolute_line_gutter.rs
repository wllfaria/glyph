use crate::config::Config;
use crate::pane::Viewport;
use crate::theme::Theme;

use crate::pane::gutter::Gutter;

#[derive(Debug)]
pub struct AbsoluteLineGutter {
    config: &'static Config,
}

impl AbsoluteLineGutter {
    pub fn new() -> Self {
        Self {
            config: Config::get(),
        }
    }
}

impl Gutter for AbsoluteLineGutter {
    fn draw(&mut self, viewport: &mut Viewport, total_lines: usize, _: u16, scroll: u16) {
        let total_lines = usize::min(viewport.dimensions.height as usize, total_lines);
        let mut scroll = scroll;
        let theme = Theme::get();

        for y in 0..total_lines {
            scroll += 1;
            let mut line = scroll.to_string();
            line = " ".repeat(self.config.gutter_width as usize - 1 - line.len()) + &line;
            line.push_str(" ");

            for (x, c) in line.chars().enumerate() {
                viewport.set_cell(x, y, c, &Theme::get().style);
            }
        }

        if total_lines < viewport.dimensions.height as usize {
            for y in total_lines..viewport.dimensions.height as usize {
                let mut line = " ".repeat(self.config.gutter_width as usize - 2);
                line.push(self.config.empty_line_char);
                line.push(' ');

                for (x, c) in line.chars().enumerate() {
                    viewport.set_cell(x, y, c, &Theme::get().style);
                }
            }
        }
    }
}
