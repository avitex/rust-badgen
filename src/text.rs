use alloc::vec::Vec;
use core::fmt;

use numtoa::NumToA;
use ttf_parser::{Font, OutlineBuilder};
use uluru::{Entry, LRUCache};

use super::Point;

const RALEWAY_LICENSE: &str = include_str!("../data/fonts/raleway/OFL.txt");
const RALEWAY_REG_DATA: &[u8] = include_bytes!("../data/fonts/raleway/Raleway-Regular.ttf");

pub fn raleway_reg_font() -> ttf_parser::Font<'static> {
    ttf_parser::Font::from_data(RALEWAY_REG_DATA, 0).unwrap()
}

pub fn font_licenses() -> &'static [&'static str] {
    &[RALEWAY_LICENSE]
}

/// Escapes bad characters for displaying within XML/HTML.
#[derive(Debug)]
pub struct Escape<'a>(pub &'a str);

#[inline]
fn escape_char(c: u8) -> Option<&'static str> {
    match c {
        b'&' => Some("&amp;"),
        b'<' => Some("&lt"),
        b'>' => Some("&gt"),
        b'"' => Some("&quot"),
        b'\'' => Some("&#39"),
        _ => None,
    }
}

impl<'a> fmt::Display for Escape<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0;
        let mut last = 0;
        for (i, c) in s.bytes().enumerate() {
            if let Some(escaped) = escape_char(c) {
                f.write_str(&s[last..i])?;
                f.write_str(escaped)?;
                last = i + 1;
            }
        }
        if last < s.len() {
            f.write_str(&s[last..])?;
        }
        Ok(())
    }
}

struct PathSink<'a> {
    scale: f32,
    last: Point<f32>,
    path: &'a mut Vec<u8>,
    i32_buf: [u8; 16],
    f32_buf: ryu::Buffer,
    integer_path: bool,
}

impl<'a> PathSink<'a> {
    fn new(scale: f32, integer_path: bool, path: &'a mut Vec<u8>) -> Self {
        Self {
            path,
            scale,
            integer_path,
            f32_buf: Default::default(),
            i32_buf: Default::default(),
            last: Point { x: 0.0, y: 0.0 },
        }
    }

    #[inline]
    fn write_str(&mut self, s: &str) {
        self.path.extend_from_slice(s.as_bytes());
    }

    #[inline]
    fn write_x(&mut self, x: f32, first: bool) {
        self.write_scaled_f32(x - self.last.x as f32, first)
    }

    #[inline]
    fn write_y(&mut self, y: f32) {
        self.write_scaled_f32(self.last.y as f32 - y, false)
    }

    #[inline]
    fn write_scaled_f32(&mut self, v: f32, first: bool) {
        self.write_f32(v * self.scale, first);
    }

    #[inline]
    fn write_f32(&mut self, v: f32, first: bool) {
        if !first && v >= 0.0 {
            self.write_str(" ");
        }
        let vi32 = v as i32;
        let bytes = if self.integer_path || v == vi32 as f32 {
            vi32.numtoa(10, &mut self.i32_buf[..])
        } else {
            self.f32_buf.format_finite(v).as_bytes()
        };
        self.path.extend_from_slice(bytes)
    }

    #[inline]
    fn write_move_to_abs(&mut self, point: Point<f32>) {
        self.write_str("M");
        self.write_f32(point.x, true);
        self.write_f32(point.y, false);
    }

    #[inline]
    fn set_last(&mut self, x: f32, y: f32) {
        self.last = Point { x, y };
    }
}

impl<'a> OutlineBuilder for PathSink<'a> {
    #[inline]
    fn move_to(&mut self, x: f32, y: f32) {
        self.write_str("m");
        self.write_x(x, true);
        self.write_y(y);
        self.set_last(x, y);
    }

    #[inline]
    fn line_to(&mut self, x: f32, y: f32) {
        self.write_str("l");
        self.write_x(x, true);
        self.write_y(y);
        self.set_last(x, y);
    }

    #[inline]
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.write_str("q");
        self.write_x(x1, true);
        self.write_y(y1);
        self.write_x(x, false);
        self.write_y(y);
        self.set_last(x, y);
    }

    #[inline]
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.write_str("c");
        self.write_x(x1, true);
        self.write_y(y1);
        self.write_x(x2, false);
        self.write_y(y2);
        self.write_x(x, false);
        self.write_y(y);
        self.set_last(x, y);
    }

    #[inline]
    fn close(&mut self) {
        self.write_str("Z");
    }
}

pub trait TextRenderer {
    fn render(&mut self, text: &str, path: &mut Vec<u8>, origin: Point) -> Option<u32>;
}

struct CacheEntry {
    c: char,
    hor_adv: f32,
    path: Vec<u8>,
}

pub struct ScaledFont<'a> {
    scale: f32,
    integer_path: bool,
    font: &'a Font<'a>,
    cache: LRUCache<[Entry<CacheEntry>; 256]>,
}

impl<'a> ScaledFont<'a> {
    pub fn new(font: &'a Font<'a>, scale: f32) -> Self {
        Self {
            scale,
            font,
            integer_path: true,
            cache: LRUCache::default(),
        }
    }

    pub fn float_path(mut self) -> Self {
        self.integer_path = false;
        self
    }
}

impl<'a> TextRenderer for ScaledFont<'a> {
    fn render(&mut self, text: &str, path: &mut Vec<u8>, origin: Point) -> Option<u32> {
        let mut sink = PathSink::new(self.scale, self.integer_path, path);
        let mut next_glyph_origin = Point {
            x: origin.x as f32,
            y: origin.y as f32,
        };
        for c in text.chars() {
            sink.set_last(0.0, 0.0);
            sink.write_move_to_abs(next_glyph_origin);

            if let Some(entry) = self.cache.find(|entry| entry.c == c) {
                sink.path.extend_from_slice(&entry.path[..]);
                next_glyph_origin.x += entry.hor_adv;
                continue;
            }

            if let Some(glyph_id) = self.font.glyph_index(c) {
                let start = sink.path.len();
                if let Some(_) = self.font.outline_glyph(glyph_id, &mut sink) {
                    let hor_adv = self.font.glyph_hor_advance(glyph_id).unwrap();
                    let hor_adv = hor_adv as f32 * self.scale;
                    next_glyph_origin.x += hor_adv;
                    let cache_entry = CacheEntry {
                        c,
                        path: sink.path[start..].to_vec(),
                        hor_adv,
                    };
                    self.cache.insert(cache_entry);
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        Some(next_glyph_origin.x as u32 - origin.x)
    }
}
