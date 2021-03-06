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

/// A badge style.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Style<'a> {
    /// The height of the badge.
    pub height: u16,
    /// The border radius of the badge.
    pub border_radius: u16,
    /// The background color of the badge.
    ///
    /// This is specific to the status.
    pub background: Color<'a>,
    /// The text color of the badge.
    pub text_color: Color<'a>,
    /// Spacing between letters.
    pub text_spacing: f32,
    // TODO: text overlay
    // pub text_overlay: bool,
    /// The text shadow color of the badge.
    pub text_shadow_color: Color<'a>,
    /// The text shadow opacity of the badge.
    pub text_shadow_opacity: Opacity<'a>,
    /// The text shadow offset of the badge.
    pub text_shadow_offset: u16,
    /// The label background color of the badge.
    pub label_background: Option<Color<'a>>,
    /// The label text color of the badge.
    ///
    /// If not `None`, defaults to `text_color`.
    pub label_text_color: Option<Color<'a>>,
    // TODO: icons
    // pub icon_path: Option<&'a str>,
    // pub icon_width: u16,
    /// The background gradient of the badge.
    pub gradient: Option<Gradient<'a>>,
}

impl<'a> Style<'a> {
    /// A classic badge style.
    pub const fn classic() -> Self {
        Self {
            height: 20,
            border_radius: 3,
            background: Color::Blue,
            // text_overlay: false,
            text_color: Color::Custom("fff"),
            text_spacing: 0.8,
            text_shadow_color: Color::Custom("000"),
            text_shadow_opacity: Opacity::raw(".25"),
            text_shadow_offset: 1,
            label_background: Some(Color::Custom("555")),
            label_text_color: None,
            // icon_path: None,
            // icon_width: 13,
            gradient: Some(Gradient {
                start: Color::Custom("eee"),
                end: None,
                opacity: Opacity::raw(".1"),
            }),
        }
    }

    /// A flat badge style.
    pub const fn flat() -> Self {
        Self {
            gradient: None,
            border_radius: 0,
            text_shadow_opacity: Opacity::raw(".1"),
            ..Self::classic()
        }
    }
}

#[inline]
fn is_valid_hex_color(hex: &str) -> bool {
    let len = hex.len();
    if len == 3 || len == 6 {
        hex.bytes().all(|b| u8::is_ascii_hexdigit(&b))
    } else {
        false
    }
}

/// Possible colors for use in a badge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color<'a> {
    /// `green`
    Green,
    /// `blue`
    Blue,
    /// `red`
    Red,
    /// `yellow`
    Yellow,
    /// `orange`
    Orange,
    /// `purple`
    Purple,
    /// `pink`
    Pink,
    /// `grey`
    Grey,
    /// `cyan`
    Cyan,
    /// `black`
    Black,
    /// A custom hex color in the form `RGB` or `RRGGBB`.
    Custom(&'a str),
}

impl<'a> Color<'a> {
    /// Parses a color value.
    ///
    /// This can be either a RGB hex value, or a named color.
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

    /// Returns a RGB hex string for the color.
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

    /// Writes the color to a [`fmt::Write`].
    #[inline]
    pub fn fmt<W>(&self, mut w: W) -> fmt::Result
    where
        W: fmt::Write,
    {
        w.write_char('#')?;
        w.write_str(self.as_str())
    }
}

/// Wrapper around a string opacity value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Opacity<'a>(&'a str);

impl<'a> Opacity<'a> {
    /// A raw unchecked opacity value.
    pub const fn raw(s: &'a str) -> Self {
        Self(s)
    }

    /// Parse an opacity value.
    pub fn parse(s: &'a str) -> Option<Self> {
        let sb = s.as_bytes();
        match sb.len() {
            1 => match sb[0] {
                b'1' | b'0' => Some(Self(s)),
                _ => None,
            },
            2 => match (sb[0], sb[1]) {
                (b'.', b'0') => Some(Self("0")),
                (b'.', c) if c.is_ascii_digit() => Some(Self(s)),
                _ => None,
            },
            3 => match (sb[0], sb[1], sb[2]) {
                (b'1', b'.', b'0') => Some(Self("1")),
                (b'.', b'0', b'0') => Some(Self("0")),
                (b'.', c, b'0') if c.is_ascii_digit() => Some(Self(&s[..2])),
                (b'0', b'.', c) if c.is_ascii_digit() => Some(Self(&s[1..])),
                (b'.', c1, c2) if c1.is_ascii_digit() && c2.is_ascii_digit() => Some(Self(s)),
                _ => None,
            },
            4 => match &s[..2] {
                "0." => Self::parse(&s[1..]),
                "1." if &s[2..] == "00" => Some(Self("1")),
                _ => None,
            },
            _ => None,
        }
    }

    /// Returns `true` if the value is completely opaque.
    #[inline]
    pub fn is_opaque(&self) -> bool {
        self.0 == "1"
    }

    /// Returns `true` if the value is completely transparent.
    #[inline]
    pub fn is_transparent(&self) -> bool {
        self.0 == "0"
    }

    /// Returns the opacity value.
    #[inline]
    pub fn as_str(&'a self) -> &str {
        self.0
    }

    /// Writes the opacity to a [`fmt::Write`].
    #[inline]
    pub fn fmt<W>(&self, mut w: W) -> fmt::Result
    where
        W: fmt::Write,
    {
        w.write_str(self.as_str())
    }
}

/// A two color gradient value.
#[derive(Debug, Clone)]
pub struct Gradient<'a> {
    /// The start color.
    pub start: Color<'a>,
    /// The end color.
    pub end: Option<Color<'a>>,
    /// The opacity of the gradient.
    pub opacity: Opacity<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opacity() {
        // Transparent
        assert_eq!(Opacity::parse("0"), Some(Opacity("0")));
        assert_eq!(Opacity::parse(".0"), Some(Opacity("0")));
        assert_eq!(Opacity::parse(".00"), Some(Opacity("0")));
        assert_eq!(Opacity::parse("0.00"), Some(Opacity("0")));
        // Opaque
        assert_eq!(Opacity::parse("1"), Some(Opacity("1")));
        assert_eq!(Opacity::parse("1.0"), Some(Opacity("1")));
        assert_eq!(Opacity::parse("1.00"), Some(Opacity("1")));
        // Fraction
        assert_eq!(Opacity::parse(".1"), Some(Opacity(".1")));
        assert_eq!(Opacity::parse("0.1"), Some(Opacity(".1")));
        assert_eq!(Opacity::parse("0.11"), Some(Opacity(".11")));
        // Invalid
        assert_eq!(Opacity::parse("2"), None);
        assert_eq!(Opacity::parse("0."), None);
        assert_eq!(Opacity::parse("1."), None);
        assert_eq!(Opacity::parse(".a"), None);
        assert_eq!(Opacity::parse("0.a"), None);
        assert_eq!(Opacity::parse("0.111"), None);
    }
}
