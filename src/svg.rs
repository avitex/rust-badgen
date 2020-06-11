use core::fmt;

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
        write!(self.w, "<{}", name)?;
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
        write!(self.w, "</{}>", name)?;
        #[cfg(feature = "pretty")]
        write!(self.w, "\n")?;
        Ok(self)
    }

    pub fn close_inline(&mut self) -> Result<&mut Self, fmt::Error> {
        assert!(self.open);
        self.open = false;
        write!(self.w, "/>")?;
        #[cfg(feature = "pretty")]
        write!(self.w, "\n")?;
        Ok(self)
    }

    pub fn attr<V>(&mut self, name: &str, value: V) -> Result<&mut Self, fmt::Error>
    where
        V: fmt::Display,
    {
        write!(self.w, r#" {}="{}""#, name, value)?;
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
            write!(self.w, "\t")?;
        }
        Ok(())
    }

    fn end_if_open(&mut self) -> fmt::Result {
        if self.open {
            write!(self.w, ">")?;
            #[cfg(feature = "pretty")]
            {
                self.indent_delta(1);
                write!(self.w, "\n")?;
            }
            self.open = false;
        }
        Ok(())
    }
}
