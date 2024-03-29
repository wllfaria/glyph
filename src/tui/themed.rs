use crossterm::style::Stylize;

use crate::theme::Style;

pub trait Themed {
    fn themed(self, style: Style) -> crossterm::style::StyledContent<String>;
}

impl Themed for String {
    fn themed(self, style: Style) -> crossterm::style::StyledContent<String> {
        let mut styled = self.stylize();

        if let Some(fg) = style.fg {
            styled = styled.with(fg);
        }

        if let Some(bg) = style.bg {
            styled = styled.on(bg);
        }

        if let Some(true) = style.italic {
            styled = styled.italic()
        }

        if let Some(true) = style.bold {
            styled = styled.bold();
        }

        styled
    }
}
