use std::io::Write;

use super::data::RenderData;

use crossterm::Result;

pub trait Margin<W> {
    fn width(&self) -> usize;
    fn draw(&mut self, write: &mut W, line_idx: usize, data: &RenderData) -> Result<()>;
}

// impl<W: Write> Margin<W>for Box<dyn Margin<W>> {
//     fn width(&self) -> usize {
//         (**self).width()
//     }
//     fn draw(&mut self, write: &mut W, line_idx: usize, data: &RenderData) -> Result<()> {
//         (**self).draw(write, line_idx, data)
//     }
// }

pub struct NoMargin;

impl<W> Margin<W> for NoMargin {
    fn width(&self) -> usize {
        0
    }

    fn draw(&mut self, _: &mut W, _: usize, _: &RenderData) -> Result<()> {
        Ok(())
    }
}

pub struct GutterMargin<'s> {
    width: usize,
    delim: &'s [&'s str; 2],
    delim_len: usize,
}

impl<'s> GutterMargin<'s> {
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

impl<W: Write> Margin<W> for GutterMargin<'_> {
    fn width(&self) -> usize {
        self.width + self.delim_len
    }

    fn draw(&mut self, write: &mut W, line_idx: usize, data: &RenderData) -> Result<()> {
        write!(write, "{:>width$}", line_idx + 1, width = self.width)?;
        if self.delim.len() != 0 {
            let z = if line_idx == data.cursor().ln { 1 } else { 0 };
            write.write(self.delim[z].as_bytes())?;
        }
        Ok(())
    }
}
