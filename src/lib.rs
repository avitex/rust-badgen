//! Rust SVG badge generator with font path rendering.
//! 
//! ```text
//! +-------+--------+
//! | LABEL | STATUS |
//! +-------+--------+
//! ```
//! 
//! # Basic usage
//! 
//! ```rust
//! let badge = badgen::badge(
//!     &badgen::Style::classic(),
//!     "status",
//!     Some("label"),
//! ).unwrap();
//! 
//! println!("{}", badge);
//! ```
//! 
//! # Performance usage
//! 
//! Note, given this example, you would clear `out` on every render.
//! 
//! ```rust
//! let font = badgen::notosans_font();
//! let mut font = badgen::font(&font);
//! let mut scratch = String::with_capacity(4098);
//! let mut out = String::with_capacity(4098);
//! 
//! badgen::write_badge_with_font(
//!     &mut out,
//!     &badgen::Style::classic(),
//!     "world",
//!     Some("hello"),
//!     &mut font,
//!     &mut scratch,
//! )
//! .unwrap();
//! 
//! println!("{}", out);
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

extern crate alloc;

mod font;
mod style;
mod svg;
mod util;

use alloc::string::String;
use core::{fmt, str};

pub use self::font::*;
pub use self::style::*;

use self::svg::SvgWrite;

const MASK_ID: &str = "m";
const GRADIENT_ID: &str = "g";
const LABEL_PATH_ID: &str = "l";
const STATUS_PATH_ID: &str = "s";

const VIEWBOX_SCALE: u32 = 100;
const VIEWBOX_ORIGIN: Point = Point { x: 0, y: 0 };

const VIEWBOX_HEIGHT: u32 = 20 * VIEWBOX_SCALE;
const SIDE_MARGIN: u32 = 5 * VIEWBOX_SCALE;
const MIDDLE_MARGIN: u32 = 11 * VIEWBOX_SCALE;
const LINE_HEIGHT: u32 = 11 * VIEWBOX_SCALE;

/// An `x` and `y` coordinate.
#[derive(Debug, Clone, Copy)]
pub struct Point<T = u32> {
    /// The `x` component.
    pub x: T,
    /// The `y` component.
    pub y: T,
}

/// Prepares a cached True Type Font for use in generating badges with integer
/// SVG paths.
pub fn font<'a>(font: &'a ttf_parser::Font<'a>) -> CachedFont<TrueTypeFont<'a>> {
    font_with_precision(font, 0)
}

/// Prepares a cached True Type Font for use in generating badges with a given
/// precision.
pub fn font_with_precision<'a>(
    font: &'a ttf_parser::Font<'a>,
    precision: u8,
) -> CachedFont<TrueTypeFont<'a>> {
    CachedFont::new(TrueTypeFont::new(font, LINE_HEIGHT as f32, precision))
}

/// Generate an SVG badge given a style, status and optional label.
///
/// Uses the default font provided by this library.
///
/// # Example
///
/// ```rust
/// let badge = badgen::badge(
///     &badgen::Style::classic(),
///     "status",
///     Some("label"),
/// ).unwrap();
///
/// println!("{}", badge);
/// ```
#[cfg(feature = "font-notosans")]
pub fn badge(style: &Style<'_>, status: &str, label: Option<&str>) -> Result<String, fmt::Error> {
    let mut out = String::with_capacity(8192);
    write_badge(&mut out, style, status, label)?;
    Ok(out)
}

/// Writes an SVG badge to a [`fmt::Write`] given a style, status and optional
/// label.
///
/// Uses the default font provided by this library.
#[cfg(feature = "font-notosans")]
pub fn write_badge<W>(
    w: &mut W,
    style: &Style<'_>,
    status: &str,
    label: Option<&str>,
) -> Result<(), fmt::Error>
where
    W: fmt::Write,
{
    let ttf_font = notosans_font();
    let mut font = font(&ttf_font);
    let mut scratch = String::with_capacity(4096);
    write_badge_with_font(w, style, status, label, &mut font, &mut scratch)
}

/// Writes an SVG badge to a [`fmt::Write`] given a style, status, optional
/// label, font and scratch space.
///
/// The scratch space is used for minimal to zero allocations with repeated use.
///
/// Prepare fonts for this function with `font` or `font_with_precision`.
pub fn write_badge_with_font<W, F>(
    w: &mut W,
    style: &Style<'_>,
    status: &str,
    label: Option<&str>,
    font: &mut F,
    scratch: &mut String,
) -> Result<(), fmt::Error>
where
    W: fmt::Write,
    F: Font,
{
    // Clear the scratch buffer from any previous run.
    scratch.clear();

    let viewbox_scale = VIEWBOX_HEIGHT as f32 / style.height as f32;
    let line_margin = (VIEWBOX_HEIGHT - font.height()) / 2;

    let mut status_path_offset = 0;
    let mut next_text_origin = Point {
        x: SIDE_MARGIN,
        y: VIEWBOX_HEIGHT - line_margin,
    };

    // If a label is specified, render and calculate the width.
    let label_width = if let Some(label) = label {
        let label_width = render_text_path(font, next_text_origin, label, scratch);
        status_path_offset += scratch.len();
        next_text_origin.x += label_width + MIDDLE_MARGIN;
        label_width
    } else {
        0
    };

    let has_label = status_path_offset > 0;

    // Render the status text path into the scratch buffer.
    let status_width = render_text_path(font, next_text_origin, status, scratch);

    // Calculate rect widths.
    let (status_rect_width, label_rect_width) = if has_label {
        let rect_margin = SIDE_MARGIN + (MIDDLE_MARGIN / 2);
        (status_width + rect_margin, label_width + rect_margin)
    } else {
        let rect_margin = SIDE_MARGIN * 2;
        (status_width + rect_margin, 0)
    };

    // Calculate the viewbox size.
    let viewbox_size = Point {
        x: status_rect_width + label_rect_width,
        y: VIEWBOX_HEIGHT,
    };

    // Calculate the image size.
    let image_size = Point {
        x: (viewbox_size.x as f32 / viewbox_scale) as u32,
        y: (viewbox_size.y as f32 / viewbox_scale) as u32,
    };

    let (label_text_path, status_text_path) = if has_label {
        let (label, status) = scratch.split_at(status_path_offset);
        (Some(label), status)
    } else {
        (None, &scratch[..])
    };

    ///////////////////////////////////////////////////////////////////////////

    let mut svg = SvgWrite::start(w)?;

    svg.attr_int("width", image_size.x)?
        .attr_int("height", image_size.y)?
        .attr_fn("viewBox", |mut w| {
            w.write_str("0 0 ")?;
            write_int(&mut w, viewbox_size.x)?;
            w.write_char(' ')?;
            write_int(&mut w, viewbox_size.y)
        })?
        .attr_str("xmlns", "http://www.w3.org/2000/svg")?;

    ///////////////////////////////////////////////////////////////////////////

    svg.open("defs")?;

    svg.open("path")?
        .attr_str("id", STATUS_PATH_ID)?
        .attr_str("d", status_text_path)?
        .close_inline()?;

    if let Some(label_text_path) = label_text_path {
        svg.open("path")?
            .attr_str("id", LABEL_PATH_ID)?
            .attr_str("d", label_text_path)?
            .close_inline()?;
    }

    svg.close("defs")?;

    ///////////////////////////////////////////////////////////////////////////

    let requires_mask = if let Some(ref gradient) = style.gradient {
        svg.open("linearGradient")?
            .attr_str("id", GRADIENT_ID)?
            .attr_str("x2", "0")?
            .attr_str("y2", "100%")?
            .open("stop")?
            .attr_str("offset", "0")?
            .attr_fn("stop-opacity", |w| write_opacity(w, gradient.opacity))?
            .attr_fn("stop-color", |w| write_color(w, gradient.start))?
            .close_inline()?
            .open("stop")?
            .attr_str("offset", "1")?
            .attr_fn("stop-opacity", |w| write_opacity(w, gradient.opacity))?;

        if let Some(end) = gradient.end {
            svg.attr_fn("stop-color", |w| write_color(w, end))?;
        }

        svg.close_inline()?.close("linearGradient")?;
        true
    } else {
        style.border_radius > 0
    };

    ///////////////////////////////////////////////////////////////////////////

    if requires_mask {
        svg.open("mask")?.attr_str("id", MASK_ID)?;

        svg.open("rect")?
            .attr_int("width", viewbox_size.x)?
            .attr_int("height", viewbox_size.y)?
            .attr_str("fill", "#fff")?;

        if style.border_radius > 0 {
            svg.attr_int("rx", style.border_radius * VIEWBOX_SCALE)?;
        }

        svg.close_inline()?
            .close("mask")?
            .open("g")?
            .attr_fn("mask", |w| write_id_url(w, MASK_ID))?;
    }

    ///////////////////////////////////////////////////////////////////////////

    if has_label {
        write_rect_path(
            &mut svg,
            VIEWBOX_ORIGIN,
            Point {
                x: label_rect_width,
                y: VIEWBOX_HEIGHT,
            },
            style
                .label_background
                .map(Fill::Color)
                .unwrap_or(Fill::None),
        )?;
    }

    write_rect_path(
        &mut svg,
        Point {
            x: label_rect_width,
            y: 0,
        },
        Point {
            x: status_rect_width,
            y: VIEWBOX_HEIGHT,
        },
        Fill::Color(style.background),
    )?;

    if style.gradient.is_some() {
        write_rect_path(
            &mut svg,
            VIEWBOX_ORIGIN,
            viewbox_size,
            Fill::Id(GRADIENT_ID),
        )?;
    }

    if requires_mask {
        svg.close("g")?;
    }

    ///////////////////////////////////////////////////////////////////////////

    if let Some(label) = label {
        let text_color = style.label_text_color.unwrap_or(style.text_color);
        write_text_path_ref(
            &mut svg,
            label,
            text_color,
            LABEL_PATH_ID,
            style.text_shadow_color,
            style.text_shadow_opacity,
            style.text_shadow_offset,
        )?;
    }

    write_text_path_ref(
        &mut svg,
        status,
        style.text_color,
        STATUS_PATH_ID,
        style.text_shadow_color,
        style.text_shadow_opacity,
        style.text_shadow_offset,
    )?;

    ///////////////////////////////////////////////////////////////////////////

    svg.finish().map(drop)
}

///////////////////////////////////////////////////////////////////////////////

enum Fill<'a> {
    None,
    Id(&'a str),
    Color(Color<'a>),
}

///////////////////////////////////////////////////////////////////////////////

// TODO: text overlay / acessibility
fn write_text_path_ref<W>(
    svg: &mut SvgWrite<W>,
    _text: &str,
    text_color: Color<'_>,
    text_path_id: &str,
    text_shadow_color: Color<'_>,
    text_shadow_opacity: Opacity<'_>,
    text_shadow_offset: u32,
) -> fmt::Result
where
    W: fmt::Write,
{
    svg.open("use")?
        .attr_fn("href", |w| write_id(w, text_path_id))?
        .attr_fn("fill", |w| write_color(w, text_shadow_color))?
        .attr_fn("opacity", |w| write_opacity(w, text_shadow_opacity))?
        .attr_fn("transform", |mut w| {
            w.write_str("translate(")?;
            write_int(&mut w, text_shadow_offset * VIEWBOX_SCALE)?;
            w.write_char(',')?;
            write_int(&mut w, text_shadow_offset * VIEWBOX_SCALE)?;
            w.write_char(')')
        })?
        .close_inline()?;

    svg.open("use")?
        .attr_fn("href", |w| write_id(w, text_path_id))?
        .attr_fn("fill", |w| write_color(w, text_color))?
        .close_inline()?;

    Ok(())
}

fn write_rect_path<W>(
    svg: &mut SvgWrite<W>,
    origin: Point,
    size: Point,
    fill: Fill<'_>,
) -> fmt::Result
where
    W: fmt::Write,
{
    svg.open("path")?.attr_fn("d", |mut w| {
        w.write_char('M')?;
        write_int(&mut w, origin.x)?;
        w.write_char(' ')?;
        write_int(&mut w, origin.y)?;
        w.write_char('h')?;
        write_int(&mut w, size.x)?;
        w.write_char('v')?;
        write_int(&mut w, size.y)?;
        w.write_char('H')?;
        write_int(&mut w, origin.x)?;
        w.write_char('z')
    })?;
    match fill {
        Fill::None => {}
        Fill::Color(c) => {
            svg.attr_fn("fill", |w| write_color(w, c))?;
        }
        Fill::Id(id) => {
            svg.attr_fn("fill", |w| write_id_url(w, id))?;
        }
    }

    svg.close_inline()?;
    Ok(())
}

#[inline]
fn write_int<W>(w: W, value: impl itoa::Integer) -> fmt::Result
where
    W: fmt::Write,
{
    itoa::fmt(w, value)
}

#[inline]
fn write_id<W>(mut w: W, id: &str) -> fmt::Result
where
    W: fmt::Write,
{
    w.write_char('#')?;
    w.write_str(id)
}

#[inline]
fn write_color<W>(w: W, color: Color<'_>) -> fmt::Result
where
    W: fmt::Write,
{
    color.fmt(w)
}

#[inline]
fn write_opacity<W>(w: W, opacity: Opacity<'_>) -> fmt::Result
where
    W: fmt::Write,
{
    opacity.fmt(w)
}

#[inline]
fn write_id_url<W>(mut w: W, id: &str) -> fmt::Result
where
    W: fmt::Write,
{
    w.write_str("url(#")?;
    w.write_str(id)?;
    w.write_char(')')
}
