//!
//! # Design
//!
//! ```text
//! +-------+--------+
//! | LABEL | STATUS |
//! +-------+--------+
//! ```
//!
//! | Field               | Value (viewbox units)                                                                   |
//! | ------------------- | --------------------------------------------------------------------------------------- |
//! | `viewbox-height`    | `2000`                                                                                  |
//! | `viewbox-width`     | `(<label-text-width> + <middle-margin> * 2)? + <status-text-width> + <side-margin> * 2` |
//! | `side-margin`       | `500`                                                                                   |
//! | `middle-margin`     | `550`                                                                                   |
//! | `line-height`       | `1100`                                                                                  |
//! | `line-margin`       | `(<viewbox-height> - <x-height>) / 2`                                                   |
//! | `label-text-width`  | Calculated on render                                                                    |
//! | `status-text-width` | Calculated on render                                                                    |

#![no_std]
#![doc(html_root_url = "https://docs.rs/badgen/0.1.0")]
// #![deny(
//     warnings,
//     missing_docs,
//     missing_debug_implementations,
//     intra_doc_link_resolution_failure,
//     rust_2018_idioms,
//     unreachable_pub
// )]

extern crate alloc;

mod style;
mod svg;
mod text;

use alloc::vec::Vec;
use core::{fmt, str};

pub use self::style::*;
pub use self::text::*;

use self::svg::SvgWrite;

const MASK_ID: &str = "m";
const GRADIENT_ID: &str = "g";
const LABEL_PATH_ID: &str = "l";
const STATUS_PATH_ID: &str = "s";

const SIDE_MARGIN: u32 = 500;
const MIDDLE_MARGIN: u32 = 1100;
const LINE_HEIGHT: u32 = 1100;
const VIEWBOX_HEIGHT: u32 = 2000;
const VIEWBOX_ORIGIN: Point = Point { x: 0, y: 0 };

#[derive(Debug, Clone, Copy)]
pub struct Point<T = u32> {
    pub x: T,
    pub y: T,
}

pub fn badge<W>(
    w: &mut W,
    style: &Style<'_>,
    status: &str,
    label: Option<&str>,
) -> Result<(), fmt::Error>
where
    W: fmt::Write,
{
    let font = raleway_reg_font();
    let scale = font.height() as f32 / LINE_HEIGHT as f32;

    let mut renderer = ScaledFont::new(&font, scale);
    let mut scratch = Vec::with_capacity(4098);
    badge_with_font(w, style, status, label, &mut renderer, &mut scratch)
}

pub fn badge_with_font<W, R>(
    w: &mut W,
    style: &Style<'_>,
    status: &str,
    label: Option<&str>,
    renderer: &mut R,
    scratch: &mut Vec<u8>,
) -> Result<(), fmt::Error>
where
    W: fmt::Write,
    R: TextRenderer,
{
    // Clear the scratch buffer from any previous run.
    scratch.clear();

    let viewbox_scale = VIEWBOX_HEIGHT as f32 / style.height as f32;
    let line_margin = (VIEWBOX_HEIGHT - renderer.x_height()) / 2;

    let mut status_path_offset = 0;
    let mut next_text_origin = Point {
        x: SIDE_MARGIN,
        y: VIEWBOX_HEIGHT - line_margin,
    };

    // If a label is specified, render and calculate the width.
    let label_width = if let Some(label) = label {
        let label_width = render_text_path(renderer, next_text_origin, label, scratch);
        status_path_offset += scratch.len();
        next_text_origin.x += label_width + MIDDLE_MARGIN;
        label_width
    } else {
        0
    };

    let has_label = status_path_offset > 0;

    // Render the status text path into the scratch buffer.
    let status_width = render_text_path(renderer, next_text_origin, status, scratch);

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

    // Get the path strings for both the label and status.
    // Note: Neither of these unwraps will never panic as the
    // strings are built safely.
    // TODO: use unsafe?
    let label_text_path = if has_label {
        Some(str::from_utf8(&scratch[..status_path_offset]).unwrap())
    } else {
        None
    };
    let status_text_path = str::from_utf8(&scratch[status_path_offset..]).unwrap();

    ///////////////////////////////////////////////////////////////////////////

    let mut svg = SvgWrite::start(w)?;

    svg.attr("width", image_size.x)?
        .attr("height", image_size.y)?
        .attr(
            "viewBox",
            format_args!("0 0 {} {}", viewbox_size.x, viewbox_size.y),
        )?
        .attr("xmlns", "http://www.w3.org/2000/svg")?;

    ///////////////////////////////////////////////////////////////////////////

    svg.open("defs")?;

    svg.open("path")?
        .attr("id", STATUS_PATH_ID)?
        .attr("d", status_text_path)?
        .close_inline()?;

    if let Some(label_text_path) = label_text_path {
        svg.open("path")?
            .attr("id", LABEL_PATH_ID)?
            .attr("d", label_text_path)?
            .close_inline()?;
    }

    svg.close("defs")?;

    ///////////////////////////////////////////////////////////////////////////

    let requires_mask = if let Some(ref gradient) = style.gradient {
        svg.open("linearGradient")?
            .attr("id", GRADIENT_ID)?
            .attr("x2", "0")?
            .attr("y2", "100%")?
            .open("stop")?
            .attr("offset", "0")?
            .attr("stop-opacity", gradient.opacity)?
            .attr("stop-color", gradient.start)?
            .close_inline()?
            .open("stop")?
            .attr("offset", "1")?
            .attr("stop-opacity", gradient.opacity)?;

        if let Some(end) = gradient.end {
            svg.attr("stop-color", end)?;
        }

        svg.close_inline()?.close("linearGradient")?;
        true
    } else {
        style.border_radius > 0
    };

    ///////////////////////////////////////////////////////////////////////////

    if requires_mask {
        svg.open("mask")?.attr("id", MASK_ID)?;

        svg.open("rect")?
            .attr("width", viewbox_size.x)?
            .attr("height", viewbox_size.y)?
            .attr("fill", "#fff")?;

        if style.border_radius > 0 {
            svg.attr("rx", style.border_radius * 10)?;
        }

        svg.close_inline()?
            .close("mask")?
            .open("g")?
            .attr("mask", format_args!("url(#{})", MASK_ID))?;
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
            style.label_background,
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
        Some(style.background),
    )?;

    if style.gradient.is_some() {
        write_rect_path(
            &mut svg,
            VIEWBOX_ORIGIN,
            viewbox_size,
            Some(format_args!("url(#{})", GRADIENT_ID)),
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
        )?;
    }

    write_text_path_ref(
        &mut svg,
        status,
        style.text_color,
        STATUS_PATH_ID,
        style.text_shadow_color,
        style.text_shadow_opacity,
    )?;

    ///////////////////////////////////////////////////////////////////////////

    svg.finish().map(drop)
}

fn render_text_path<T: TextRenderer>(
    renderer: &mut T,
    origin: Point,
    text: &str,
    buf: &mut Vec<u8>,
) -> u32 {
    // TODO
    renderer.render(text, buf, origin).unwrap()
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
) -> fmt::Result
where
    W: fmt::Write,
{
    svg.open("use")?
        .attr("href", format_args!("#{}", text_path_id))?
        .attr("fill", text_shadow_color)?
        .attr("opacity", text_shadow_opacity)?
        .attr("transform", "translate(100,100)")?
        .close_inline()?;

    svg.open("use")?
        .attr("href", format_args!("#{}", text_path_id))?
        .attr("fill", text_color)?
        .close_inline()?;

    Ok(())
}

fn write_rect_path<W, F>(
    svg: &mut SvgWrite<W>,
    origin: Point,
    size: Point,
    fill: Option<F>,
) -> fmt::Result
where
    W: fmt::Write,
    F: fmt::Display,
{
    svg.open("path")?.attr(
        "d",
        format_args!(
            "M{origin_x} {origin_y}h{size_x}v{size_y}H{origin_x}z",
            origin_x = origin.x,
            origin_y = origin.y,
            size_x = size.x,
            size_y = size.y,
        ),
    )?;
    if let Some(fill) = fill {
        svg.attr("fill", fill)?;
    }
    svg.close_inline()?;
    Ok(())
}
