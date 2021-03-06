use alloc::string::String;

use ttf_parser::{Face as TrueTypeFontInner, OutlineBuilder};
use uluru::LRUCache;

use super::Point;

#[cfg(feature = "font-noto-sans")]
const NOTOSANS_LICENSE: &str = include_str!("../data/fonts/notosans/LICENSE.txt");
#[cfg(feature = "font-noto-sans")]
const NOTOSANS_DATA: &[u8] = include_bytes!("../data/fonts/notosans/NotoSans-Regular.ttf");

/// Parsed NotoSans font.
#[cfg(feature = "font-noto-sans")]
pub fn notosans_font() -> ttf_parser::Face<'static> {
    ttf_parser::Face::from_slice(NOTOSANS_DATA, 0).unwrap()
}

/// Licenses for fonts included in binary.
pub fn font_licenses() -> &'static [&'static str] {
    &[
        #[cfg(feature = "font-noto-sans")]
        NOTOSANS_LICENSE,
    ]
}

///////////////////////////////////////////////////////////////////////////////

/// A Font trait specific to badge generation.
pub trait Font {
    /// The font height relative to the badge viewbox.
    fn height(&self) -> u32;

    /// Render a character glyph if it exists.
    fn render_glyph(&mut self, c: char) -> Option<FontGlyph<'_>>;

    /// The scale of the font used in path rendering.
    fn scale(&self) -> f32 {
        1.0
    }

    /// The precision of the font used in path rendering.
    fn precision(&self) -> u8 {
        1
    }
}

/// A rendered glyph.
#[derive(Debug)]
pub struct FontGlyph<'a> {
    /// The path of the glyph, if it has one.
    pub path: Option<&'a str>,
    /// The horizontal advance of the glyph.
    pub hor_advance: f32,
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
struct CachedGlyph {
    path: Option<String>,
    character: char,
    hor_advance: f32,
}

/// A wrapper around a font that caches a finite number of glyph paths.
#[derive(Debug, Clone)]
pub struct CachedFont<T> {
    font: T,
    cache: LRUCache<CachedGlyph, 256>,
}

impl<T> CachedFont<T> {
    /// Construct a new [`CachedFont`].
    pub fn new(font: T) -> Self {
        Self {
            font,
            cache: Default::default(),
        }
    }
}

impl<T> Font for CachedFont<T>
where
    T: Font,
{
    fn height(&self) -> u32 {
        self.font.height()
    }

    fn render_glyph(&mut self, c: char) -> Option<FontGlyph<'_>> {
        if self.cache.touch(|entry| entry.character == c) {
            return self.cache.front().map(|entry| FontGlyph {
                path: entry.path.as_deref(),
                hor_advance: entry.hor_advance,
            });
        }

        match self.font.render_glyph(c) {
            Some(glyph) => {
                self.cache.insert(CachedGlyph {
                    character: c,
                    path: glyph.path.map(String::from),
                    hor_advance: glyph.hor_advance,
                });
                Some(glyph)
            }
            None => None,
        }
    }

    fn scale(&self) -> f32 {
        self.font.scale()
    }

    fn precision(&self) -> u8 {
        self.font.precision()
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A True Type Font with a font height and path precision.
#[derive(Debug, Clone)]
pub struct TrueTypeFont<'a> {
    font: &'a TrueTypeFontInner<'a>,
    scale: f32,
    height: u32,
    precision: u8,
    path_buffer: String,
}

impl<'a> TrueTypeFont<'a> {
    /// Construct a new [`TrueTypeFont`].
    pub fn new(font: &'a TrueTypeFontInner<'a>, font_height: f32, precision: u8) -> Self {
        let units_per_em = font.units_per_em().expect("units-per-em not found") as f32;
        let scale = font_height / units_per_em;
        let height = font_height + (font.descender() as f32 * scale);

        Self {
            font,
            scale,
            precision,
            height: height as u32,
            path_buffer: String::default(),
        }
    }
}

impl<'a> Font for TrueTypeFont<'a> {
    fn height(&self) -> u32 {
        self.height
    }

    fn render_glyph(&mut self, c: char) -> Option<FontGlyph<'_>> {
        self.path_buffer.clear();
        let mut sink = PathSink::new(self.scale, self.precision, &mut self.path_buffer);
        if let Some(glyph_id) = self.font.glyph_index(c) {
            let hor_advance = self.font.glyph_hor_advance(glyph_id).unwrap();
            let hor_advance = hor_advance as f32 * self.scale;
            let path = match self.font.outline_glyph(glyph_id, &mut sink) {
                Some(_) => Some(self.path_buffer.as_str()),
                None => None,
            };
            return Some(FontGlyph { path, hor_advance });
        }
        None
    }

    fn scale(&self) -> f32 {
        self.scale
    }

    fn precision(&self) -> u8 {
        self.precision
    }
}

///////////////////////////////////////////////////////////////////////////////

struct PathSink<'a> {
    scale: f32,
    last: Point<f32>,
    path: &'a mut String,
    f32_buf: ryu::Buffer,
    precision: u8,
    precision_mod: f32,
}

impl<'a> PathSink<'a> {
    fn new(scale: f32, precision: u8, path: &'a mut String) -> Self {
        let precision_mod = if precision == 0 {
            1.0
        } else {
            precision as f32 * 10.0
        };
        Self {
            path,
            scale,
            precision,
            precision_mod,
            f32_buf: Default::default(),
            last: Point { x: 0.0, y: 0.0 },
        }
    }

    #[inline]
    fn write_str(&mut self, s: &str) {
        self.path.push_str(s);
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
    fn write_f32(&mut self, mut v: f32, first: bool) {
        v = (v * self.precision_mod).round() / self.precision_mod;
        if !first && v >= 0.0 {
            self.write_str(" ");
        }
        let vi32 = v as i32;
        if self.precision == 0 || (v - vi32 as f32).abs() < f32::EPSILON {
            itoa::fmt(&mut self.path, vi32).ok();
        } else {
            let s = self.f32_buf.format_finite(v);
            self.path.push_str(s)
        }
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

///////////////////////////////////////////////////////////////////////////////

pub(crate) fn render_text_path<T: Font>(
    font: &mut T,
    origin: Point,
    text: &str,
    letter_spacing: f32,
    path_buffer: &mut String,
) -> u32 {
    let mut sink = PathSink::new(font.scale(), font.precision(), path_buffer);
    let letter_spacing = letter_spacing * font.scale();

    let mut next_glyph_origin = Point {
        x: origin.x as f32 + letter_spacing,
        y: origin.y as f32,
    };

    for c in text.chars() {
        // TODO: can't render?
        if let Some(entry) = font.render_glyph(c) {
            if let Some(path) = entry.path {
                sink.set_last(0.0, 0.0);
                sink.write_move_to_abs(next_glyph_origin);
                sink.write_str(path);
            }
            next_glyph_origin.x += entry.hor_advance + letter_spacing;
        }
    }

    next_glyph_origin.x as u32 - origin.x
}
