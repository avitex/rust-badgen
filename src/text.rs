const VERDANA_110_TABLE: &[u8] = include_bytes!("../font-widths/verdana110.bin");

const BAD_CHARS: &[u8] = &[b'&', b'<'];

pub(crate) const VERDANA_110_CHAR_WIDTHS: CharWidths = CharWidths {
    table: VERDANA_110_TABLE,
    // Width as "@" for overflows
    fallback: VERDANA_110_TABLE[64],
};

#[derive(Debug)]
pub struct InvalidChar;

#[derive(Debug, Clone, Copy)]
pub(crate) struct CharWidths {
    table: &'static [u8],
    fallback: u8,
}

impl CharWidths {
    pub(crate) fn text_width(&self, text: &str) -> u32 {
        text.chars().map(|c| self.char_width(c)).sum()
    }

    pub(crate) fn char_width(&self, chr: char) -> u32 {
        if (chr as usize) < self.table.len() {
            self.table[chr as usize] as u32
        } else {
            self.fallback as u32
        }
    }
}

pub(crate) fn validate_text(s: &str) -> Result<(), InvalidChar> {
    if s.bytes().any(|b| BAD_CHARS.contains(&b)) {
        Err(InvalidChar)
    } else {
        Ok(())
    }
}
