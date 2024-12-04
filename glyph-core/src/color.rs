use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0} is not a valid color name")]
    InvalidName(String),
    #[error("{0} is not a valid color hex")]
    InvalidHex(String),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    #[default]
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    LightGray,
    White,
    Rgb(u8, u8, u8),
    Indexed(u8),
}

fn from_str(value: &str) -> Result<Color, Error> {
    match value.to_lowercase().as_str() {
        "reset" => Ok(Color::Reset),
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "magenta" => Ok(Color::Magenta),
        "cyan" => Ok(Color::Cyan),
        "gray" => Ok(Color::Gray),
        "light_red" => Ok(Color::LightRed),
        "light_green" => Ok(Color::LightGreen),
        "light_yellow" => Ok(Color::LightYellow),
        "light_blue" => Ok(Color::LightBlue),
        "light_magenta" => Ok(Color::LightMagenta),
        "light_cyan" => Ok(Color::LightCyan),
        "light_gray" => Ok(Color::LightGray),
        "white" => Ok(Color::White),
        _ if value.starts_with("#") => try_parse_hex(value),
        _ => Err(Error::InvalidName(value.to_string())),
    }
}

impl TryFrom<&str> for Color {
    type Error = Error;

    fn try_from(value: &str) -> Result<Color, Self::Error> {
        from_str(value)
    }
}

impl TryFrom<String> for Color {
    type Error = Error;

    fn try_from(value: String) -> Result<Color, Self::Error> {
        from_str(&value)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Color {
        Color::Rgb(r, g, b)
    }
}

fn try_parse_hex(value: &str) -> Result<Color, Error> {
    let hex_str = &value[1..];

    if hex_str.len() < 6 || hex_str.len() > 6 {
        return Err(Error::InvalidHex(value.to_string()));
    }

    let r = &hex_str[..2];
    let g = &hex_str[2..4];
    let b = &hex_str[4..6];

    let r = u8::from_str_radix(r, 16).map_err(|_| Error::InvalidHex(value.to_string()))?;
    let g = u8::from_str_radix(g, 16).map_err(|_| Error::InvalidHex(value.to_string()))?;
    let b = u8::from_str_radix(b, 16).map_err(|_| Error::InvalidHex(value.to_string()))?;

    Ok(Color::Rgb(r, g, b))
}
