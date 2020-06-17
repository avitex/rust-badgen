use core::fmt;

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

impl<'a> Escape<'a> {
    pub fn fmt<W>(&self, mut w: W) -> fmt::Result
    where
        W: fmt::Write,
    {
        let s = self.0;
        let mut last = 0;
        for (i, c) in s.bytes().enumerate() {
            if let Some(escaped) = escape_char(c) {
                w.write_str(&s[last..i])?;
                w.write_str(escaped)?;
                last = i + 1;
            }
        }
        if last < s.len() {
            w.write_str(&s[last..])?;
        }
        Ok(())
    }
}
