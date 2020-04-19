use core::fmt;

pub(crate) const GREEN_COLOR_HEX: &str = "3C1";
pub(crate) const BLUE_COLOR_HEX: &str = "08C";
pub(crate) const RED_COLOR_HEX: &str = "E43";
pub(crate) const YELLOW_COLOR_HEX: &str = "DB1";
pub(crate) const ORANGE_COLOR_HEX: &str = "F73";
pub(crate) const PURPLE_COLOR_HEX: &str = "94E";
pub(crate) const PINK_COLOR_HEX: &str = "E5B";
pub(crate) const GREY_COLOR_HEX: &str = "999";
pub(crate) const CYAN_COLOR_HEX: &str = "1BC";
pub(crate) const BLACK_COLOR_HEX: &str = "2A2A2A";

#[inline]
fn is_valid_hex_color(hex: &str) -> bool {
    let len = hex.len();
    if len == 3 || len == 6 {
        hex.bytes().all(|b| u8::is_ascii_hexdigit(&b))
    } else {
        false
    }
}

#[derive(Debug, PartialEq)]
pub enum Color<'a> {
    Green,
    Blue,
    Red,
    Yellow,
    Orange,
    Purple,
    Pink,
    Grey,
    Cyan,
    Black,
    Custom(&'a str),
}

impl<'a> Color<'a> {
    #[inline]
    pub fn parse(s: &'a str) -> Option<Self> {
        let color = match s {
            "green" | "GREEN" => Self::Green,
            "blue" | "BLUE" => Self::Blue,
            "red" | "RED" => Self::Red,
            "yellow" | "YELLOW" => Self::Yellow,
            "orange" | "ORANGE" => Self::Orange,
            "purple" | "PURPLE" => Self::Purple,
            "pink" | "PINK" => Self::Pink,
            "grey" | "GREY" | "gray" | "GRAY" => Self::Grey,
            "cyan" | "CYAN" => Self::Cyan,
            "black" | "BLACK" => Self::Black,
            other if is_valid_hex_color(other) => Self::Custom(other),
            _ => return None,
        };
        Some(color)
    }

    #[inline]
    pub fn as_str(&'a self) -> &str {
        match self {
            Self::Green => GREEN_COLOR_HEX,
            Self::Blue => BLUE_COLOR_HEX,
            Self::Red => RED_COLOR_HEX,
            Self::Yellow => YELLOW_COLOR_HEX,
            Self::Orange => ORANGE_COLOR_HEX,
            Self::Purple => PURPLE_COLOR_HEX,
            Self::Pink => PINK_COLOR_HEX,
            Self::Grey => GREY_COLOR_HEX,
            Self::Cyan => CYAN_COLOR_HEX,
            Self::Black => BLACK_COLOR_HEX,
            Self::Custom(s) => s,
        }
    }
}

impl<'a> fmt::Display for Color<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.as_str())
    }
}
