use std::io::Write;

use super::data::RenderData;

use crossterm::Result;

pub trait Margin<W> {
    fn width(&self) -> usize;
    fn draw_margin(&mut self, write: &mut W, line_idx: usize, data: &RenderData) -> Result<()>;
}

pub struct NoGutter;

impl<W: Write> Margin<W> for NoGutter {
    fn width(&self) -> usize {
        0
    }

    fn draw_margin(&mut self, _: &mut W, _: usize, _: &RenderData) -> Result<()> {
        Ok(())
    }
}

pub struct SimpleGutter<'s> {
    width: usize,
    delim: &'s [&'s str; 2],
    delim_len: usize,
}

impl<'s> SimpleGutter<'s> {
    pub fn new(width: usize) -> Self {
        Self::with_delim(width, &[" │ ", " ┃ "])
    }

    pub fn with_delim(width: usize, delim: &'s [&'s str; 2]) -> Self {
        assert!(
            delim[0].len() == delim[1].len(),
            "All delimiters must be the same size"
        );
        Self {
            width,
            delim,
            delim_len: delim[0].chars().count(),
        }
    }
}

impl<W: Write> Margin<W> for SimpleGutter<'_> {
    fn width(&self) -> usize {
        self.width + self.delim_len
    }

    fn draw_margin(&mut self, write: &mut W, line_idx: usize, data: &RenderData) -> Result<()> {
        write!(write, "{:>width$}", line_idx + 1, width = self.width)?;
        if self.delim.len() != 0 {
            let z = if line_idx == data.cursor().ln { 1 } else { 0 };
            write.write(self.delim[z].as_bytes())?;
        }
        Ok(())
    }
}
