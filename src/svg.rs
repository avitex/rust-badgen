use core::fmt;

use crate::util::Escape;

pub struct SvgWrite<W> {
    w: W,
    open: bool,
    #[cfg(feature = "pretty")]
    level: i8,
}

impl<W> SvgWrite<W>
where
    W: fmt::Write,
{
    pub fn start(w: W) -> Result<Self, fmt::Error> {
        let mut this = Self {
            w,
            open: false,
            #[cfg(feature = "pretty")]
            level: 0,
        };
        this.open("svg")?;
        Ok(this)
    }

    pub fn open(&mut self, name: &str) -> Result<&mut Self, fmt::Error> {
        self.end_if_open()?;
        #[cfg(feature = "pretty")]
        self.write_indent()?;
        self.w.write_str("<")?;
        self.w.write_str(name)?;
        self.open = true;
        Ok(self)
    }

    pub fn close(&mut self, name: &str) -> Result<&mut Self, fmt::Error> {
        self.end_if_open()?;
        #[cfg(feature = "pretty")]
        {
            self.indent_delta(-1);
            self.write_indent()?;
        }
        self.w.write_str("</")?;
        self.w.write_str(name)?;
        self.w.write_str(">")?;
        #[cfg(feature = "pretty")]
        self.w.write_char('\n')?;
        Ok(self)
    }

    pub fn close_inline(&mut self) -> Result<&mut Self, fmt::Error> {
        assert!(self.open);
        self.open = false;
        self.w.write_str("/>")?;
        #[cfg(feature = "pretty")]
        self.w.write_char('\n')?;
        Ok(self)
    }

    #[inline]
    pub fn attr_fn<F>(&mut self, name: &str, value_fn: F) -> Result<&mut Self, fmt::Error>
    where
        F: FnOnce(&mut W) -> fmt::Result,
    {
        self.w.write_char(' ')?;
        self.w.write_str(name)?;
        self.w.write_str(r#"=""#)?;
        value_fn(&mut self.w)?;
        self.w.write_char('"')?;
        Ok(self)
    }

    pub fn attr_str(&mut self, name: &str, value: &str) -> Result<&mut Self, fmt::Error> {
        self.attr_fn(name, |w| w.write_str(value))?;
        Ok(self)
    }

    pub fn attr_int<V>(&mut self, name: &str, value: V) -> Result<&mut Self, fmt::Error>
    where
        V: itoa::Integer,
    {
        self.attr_fn(name, |w| itoa::fmt(w, value))?;
        Ok(self)
    }

    #[allow(dead_code)]
    pub fn write_value(&mut self, value: &str) -> Result<&mut Self, fmt::Error> {
        Escape(value).fmt(&mut self.w)?;
        self.end_if_open()?;
        Ok(self)
    }

    pub fn finish(mut self) -> Result<W, fmt::Error> {
        self.close("svg")?;
        Ok(self.w)
    }

    #[cfg(feature = "pretty")]
    fn indent_delta(&mut self, delta: i8) {
        self.level += delta;
    }

    #[cfg(feature = "pretty")]
    #[inline]
    fn write_indent(&mut self) -> fmt::Result {
        for _ in 0..self.level {
            self.w.write_char('\t')?;
        }
        Ok(())
    }

    fn end_if_open(&mut self) -> fmt::Result {
        if self.open {
            self.w.write_char('>')?;
            #[cfg(feature = "pretty")]
            {
                self.indent_delta(1);
                self.w.write_char('\n')?;
            }
            self.open = false;
        }
        Ok(())
    }
}
