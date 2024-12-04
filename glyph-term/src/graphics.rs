use glyph_core::color::Color;

pub trait IntoColor {
    fn into_color(self) -> crossterm::style::Color;
}

impl IntoColor for Color {
    fn into_color(self) -> crossterm::style::Color {
        use crossterm::style::Color as CColor;

        match self {
            Color::Reset => CColor::Reset,
            Color::Black => CColor::Black,
            Color::Red => CColor::DarkRed,
            Color::Green => CColor::DarkGreen,
            Color::Yellow => CColor::DarkYellow,
            Color::Blue => CColor::DarkBlue,
            Color::Magenta => CColor::DarkMagenta,
            Color::Cyan => CColor::DarkCyan,
            Color::Gray => CColor::DarkGrey,
            Color::LightRed => CColor::Red,
            Color::LightGreen => CColor::Green,
            Color::LightBlue => CColor::Blue,
            Color::LightYellow => CColor::Yellow,
            Color::LightMagenta => CColor::Magenta,
            Color::LightCyan => CColor::Cyan,
            Color::LightGray => CColor::Grey,
            Color::White => CColor::White,
            Color::Indexed(i) => CColor::AnsiValue(i),
            Color::Rgb(r, g, b) => CColor::Rgb { r, g, b },
        }
    }
}
