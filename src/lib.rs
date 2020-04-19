//! `badgen` provides badge generation.
//!
//! ## Classic badge
//! ```rust
//! let badge = badgen::Builder::new("downloads", "12").build().unwrap();
//! println!("{}", badge);
//! ```
//!
//! ## Flat badge
//! ```rust
//! let badge = badgen::Builder::new("downloads", "12").flat().build().unwrap();
//! println!("{}", badge);
//! ```

#![no_std]
#![doc(html_root_url = "https://docs.rs/badgen/0.1.0")]
#![deny(
    warnings,
    missing_docs,
    missing_debug_implementations,
    intra_doc_link_resolution_failure,
    rust_2018_idioms,
    unreachable_pub
)]

use core::fmt;
use core::str::FromStr;

mod color;
mod text;

pub use self::color::Color;
pub use self::text::InvalidChar;

use self::text::*;

const DEFAULT_TEXT_COLOR: Color<'static> = Color::Custom("fff");
const DEFAULT_TEXT_SHADOW_COLOR: Color<'static> = Color::Custom("000");
const DEFAULT_ICON_WIDTH: u32 = 13;
const DEFAULT_COLOR: Color<'static> = Color::Blue;
const DEFAULT_LABEL_COLOR: Color<'static> = Color::Custom("555");

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Style {
    Flat,
    Classic,
}

impl Default for Style {
    fn default() -> Self {
        Self::Classic
    }
}

#[derive(Debug)]
pub struct UnknownStyle;

impl FromStr for Style {
    type Err = UnknownStyle;

    fn from_str(s: &str) -> Result<Self, UnknownStyle> {
        match s {
            "classic" | "CLASSIC" => Ok(Self::Classic),
            "flat" | "FLAT" => Ok(Self::Flat),
            _ => Err(UnknownStyle),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Builder<'a> {
    status: &'a str,
    color: Color<'a>,
    label: Option<&'a str>,
    label_color: Color<'a>,
    style: Style,
    icon: Option<&'a str>,
    icon_width: u32,
    scale: f32,
}

impl<'a> Builder<'a> {
    pub fn new(label: &'a str, status: &'a str) -> Self {
        Self {
            status,
            color: DEFAULT_COLOR,
            label: Some(label),
            label_color: DEFAULT_LABEL_COLOR,
            style: Style::default(),
            icon: None,
            icon_width: DEFAULT_ICON_WIDTH,
            scale: 1.0,
        }
    }

    pub fn new_status(status: &'a str) -> Self {
        Self {
            status,
            color: DEFAULT_COLOR,
            label: None,
            label_color: DEFAULT_LABEL_COLOR,
            style: Style::default(),
            icon: None,
            icon_width: DEFAULT_ICON_WIDTH,
            scale: 1.0,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn flat(mut self) -> Self {
        self.style = Style::Flat;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn build(self) -> Result<Badge<'a>, InvalidChar> {
        validate_text(self.status)?;
        if let Some(label) = self.label {
            validate_text(label)?;
        }
        Ok(Badge::new(self))
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Badge<'a> {
    inner: BadgeInner<'a>,
}

#[derive(Debug)]
enum BadgeInner<'a> {
    Bare(BareBadge<'a>),
    Builtin(BuiltinBadge<'a>),
}

impl<'a> Badge<'a> {
    fn new(options: Builder<'a>) -> Self {
        let text_widths = VERDANA_110_CHAR_WIDTHS;
        let text_color = DEFAULT_TEXT_COLOR;
        let text_shadow_color = DEFAULT_TEXT_SHADOW_COLOR;
        let text_shadow_opacity = match options.style {
            Style::Classic => 0.25,
            Style::Flat => 0.1,
        };
        let icon_width = options.icon_width.saturating_mul(10);
        let status_text_width = text_widths.text_width(options.status);
        let (label, label_text_width, icon_span_width) = match (options.label, options.icon) {
            (None, None) => {
                let status_rect_width = status_text_width.saturating_add(115);
                let svg_width = status_rect_width / 10;
                let svg_height = 20;
                let inner = BadgeInner::Bare(BareBadge {
                    svg_width,
                    svg_height,
                    width: status_rect_width,
                    style: options.style,
                    color: options.color,
                    status: options.status,
                    status_text_width,
                    status_rect_width,
                    text_color,
                    text_shadow_color,
                    text_shadow_opacity,
                });
                return Self { inner };
            }
            (Some(label), Some(_)) => (
                label,
                text_widths.text_width(label),
                icon_width.saturating_add(30),
            ),
            (None, Some(_)) => ("", 0, icon_width.saturating_sub(18)),
            (Some(label), None) => (label, text_widths.text_width(label), 0),
        };
        let label_text_start = icon_span_width + 50;
        let label_rect_width = label_text_width + 100 + icon_span_width;
        let status_rect_width = status_text_width + 100;
        let width = label_rect_width + status_rect_width;
        let svg_width = (options.scale * width as f32 / 10.0) as u32;
        let svg_height = (options.scale * 20.0) as u32;
        let inner = BadgeInner::Builtin(BuiltinBadge {
            style: options.style,
            width,
            svg_width,
            svg_height,
            icon: options.icon,
            icon_width: options.icon_width,
            status_color: options.color,
            label_color: options.label_color,
            label,
            label_text_width,
            label_text_start,
            label_rect_width,
            status: options.status,
            status_rect_width,
            status_text_width,
            text_color,
            text_shadow_color,
            text_shadow_opacity,
        });
        Self { inner }
    }
}

impl<'a> fmt::Display for Badge<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            BadgeInner::Bare(badge) => badge.fmt(f),
            BadgeInner::Builtin(badge) => badge.fmt(f),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct BareBadge<'a> {
    style: Style,
    color: Color<'a>,
    status: &'a str,
    width: u32,
    svg_width: u32,
    svg_height: u32,
    text_color: Color<'a>,
    status_rect_width: u32,
    status_text_width: u32,
    text_shadow_opacity: f32,
    text_shadow_color: Color<'a>,
}

impl<'a> fmt::Display for BareBadge<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_svg_open(f, self.svg_width, self.svg_height, self.width, false)?;
        match self.style {
            Style::Classic => {
                write_linear_gradient(f, self.status_rect_width)?;
                write!(
                    f,
                    concat!(
                        r##"<g mask="url(#m)">"##,
                        r####"<rect width="{status_rect_width}" height="200" fill="{color}" x="0"/>"####,
                        r####"<rect width="{status_rect_width}" height="200" fill="url(#a)"/>"####,
                        r##"</g>"##,
                    ),
                    color = self.color,
                    status_rect_width = self.status_rect_width,
                )?;
            }
            Style::Flat => {
                write!(
                    f,
                    concat!(
                        r##"<g>"##,
                        r####"<rect fill="{color}" x="0" width="{status_rect_width}" height="200"/>"####,
                        r##"</g>"##,
                    ),
                    color = self.color,
                    status_rect_width = self.status_rect_width,
                )?;
            }
        }
        write!(
            f,
            concat!(
                r##"<g fill="{text_color}" text-anchor="start" font-family="Verdana,DejaVu Sans,sans-serif" font-size="110">"##,
                r####"<text x="65" y="148" textLength="{status_text_width}" fill="{text_shadow_color}" opacity="{text_shadow_opacity}">{status}</text>"####,
                r####"<text x="55" y="138" textLength="{status_text_width}">{status}</text>"####,
                r##"</g>"##,
            ),
            status = self.status,
            text_color = self.text_color,
            text_shadow_color = self.text_shadow_color,
            text_shadow_opacity = self.text_shadow_opacity,
            status_text_width = self.status_text_width,
        )?;
        write_svg_close(f)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct BuiltinBadge<'a> {
    style: Style,
    icon: Option<&'a str>,
    icon_width: u32,
    width: u32,
    svg_width: u32,
    svg_height: u32,
    label: &'a str,
    label_color: Color<'a>,
    label_text_start: u32,
    label_text_width: u32,
    label_rect_width: u32,
    status: &'a str,
    status_color: Color<'a>,
    status_rect_width: u32,
    status_text_width: u32,
    text_color: Color<'a>,
    text_shadow_color: Color<'a>,
    text_shadow_opacity: f32,
}

impl<'a> fmt::Display for BuiltinBadge<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_svg_open(
            f,
            self.svg_width,
            self.svg_height,
            self.width,
            self.icon.is_some(),
        )?;
        match self.style {
            Style::Flat => write!(
                f,
                concat!(
                    r##"<g>"##,
                    r####"<rect fill="{label_color}" width="{label_rect_width}" height="200"/>"####,
                    r####"<rect fill="{status_color}" width="{status_rect_width}" height="200" x="{label_rect_width}"/>"####,
                    r##"</g>"##,
                ),
                label_color = self.label_color,
                status_color = self.status_color,
                label_rect_width = self.label_rect_width,
                status_rect_width = self.status_rect_width,
            )?,
            Style::Classic => {
                write_linear_gradient(f, self.width)?;
                write!(
                    f,
                    concat!(
                        r##"<g mask="url(#m)">"##,
                        r####"<rect width="{label_rect_width}" height="200" fill="{label_color}"/>"####,
                        r####"<rect width="{status_rect_width}" height="200" fill="{status_color}" x="{label_rect_width}"/>"####,
                        r####"<rect width="{width}" height="200" fill="url(#a)"/>"####,
                        r##"</g>"##,
                    ),
                    width = self.width,
                    status_color = self.status_color,
                    label_color = self.label_color,
                    label_rect_width = self.label_rect_width,
                    status_rect_width = self.status_rect_width,
                )?;
            }
        };
        write!(
            f,
            concat!(
                r##"<g fill="{text_color}" text-anchor="start" font-family="Verdana,DejaVu Sans,sans-serif" font-size="110">"##,
                r####"<text x="{label_text_shadow_start}" y="148" textLength="{label_text_width}" fill="{text_shadow_color}" opacity="{text_shadow_opacity}">{label}</text>"####,
                r####"<text x="{label_text_start}" y="138" textLength="{label_text_width}">{label}</text>"####,
                r####"<text x="{status_text_shadow_start}" y="148" textLength="{status_text_width}" fill="{text_shadow_color}" opacity="{text_shadow_opacity}">{status}</text>"####,
                r####"<text x="{status_text_start}" y="138" textLength="{status_text_width}">{status}</text>"####,
                r##"</g>"##,
            ),
            label = self.label,
            label_text_start = self.label_text_start,
            label_text_shadow_start = self.label_text_start + 10,
            label_text_width = self.label_text_width,
            status = self.status,
            status_text_width = self.status_text_width,
            status_text_start = self.label_rect_width + 45,
            status_text_shadow_start = self.label_rect_width + 55,
            text_color = self.text_color,
            text_shadow_opacity = self.text_shadow_opacity,
            text_shadow_color = self.text_shadow_color,
        )?;
        if let Some(icon) = self.icon {
            write_icon(f, icon, self.icon_width)?;
        }
        write_svg_close(f)
    }
}

fn write_svg_open(
    f: &mut fmt::Formatter<'_>,
    width: u32,
    height: u32,
    viewbox_width: u32,
    xlink: bool,
) -> fmt::Result {
    write!(
        f,
        r#"<svg width="{width}" height="{height}" viewBox="0 0 {viewbox_width} 200" xmlns="http://www.w3.org/2000/svg""#,
        width = width,
        height = height,
        viewbox_width = viewbox_width,
    )?;
    if xlink {
        write!(f, r#" xmlns:xlink="http://www.w3.org/1999/xlink">"#)
    } else {
        write!(f, r#">"#)
    }
}

fn write_svg_close(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "</svg>")
}

fn write_icon(f: &mut fmt::Formatter<'_>, icon: &str, icon_width: u32) -> fmt::Result {
    write!(
        f,
        r#"<image x="40" y="35" width="{icon_width}" height="132" xlink:href="{icon}"/>"#,
        icon = icon,
        icon_width = icon_width
    )
}

fn write_linear_gradient(f: &mut fmt::Formatter<'_>, mask_width: u32) -> fmt::Result {
    write!(
        f,
        concat!(
            r##"<linearGradient id="a" x2="0" y2="100%">"##,
            r####"<stop offset="0" stop-opacity=".1" stop-color="#EEE"/>"####,
            r####"<stop offset="1" stop-opacity=".1"/>"####,
            r##"</linearGradient>"##,
            r##"<mask id="m"><rect width="{mask_width}" height="200" rx="30" fill="#FFF"/></mask>"##,
        ),
        mask_width = mask_width,
    )
}
