use std::io::Write;

use super::data::RenderData;

use crate::Result;

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

pub struct ClassicGutter;

impl ClassicGutter {
    const WIDTH: usize = 5;
    const PAD: usize = 3;

    const DELIM: &'static str = " │ ";
    const DELIM_BOLD: &'static str = " ┃ ";
}

impl<W: Write> Margin<W> for ClassicGutter {
    fn width(&self) -> usize {
        Self::WIDTH + Self::PAD
    }

    fn draw(&mut self, write: &mut W, line_idx: usize, data: &RenderData) -> Result<()> {
        write!(write, "{:>width$}", line_idx + 1, width = 5)?;

        write.write(
            if line_idx == data.cursor().ln {
                Self::DELIM_BOLD
            } else {
                Self::DELIM
            }
            .as_bytes(),
        )?;

        Ok(())
    }
}
