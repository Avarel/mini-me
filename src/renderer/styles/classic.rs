use std::io::Write;

use super::{Footer, Header, Margin};
use crate::{renderer::RenderData, Result};
use crossterm::{
    terminal::{Clear, ClearType},
    QueueableCommand,
};

pub struct ClassicHeader<'s> {
    pub message: &'s str
}

impl<W: Write> Header<W> for ClassicHeader<'_> {
    fn rows(&self) -> usize {
        1
    }

    fn draw(&mut self, w: &mut W, _: &RenderData) -> Result<()> {
        w.write("      ╭─── ".as_bytes())?;
        w.write(self.message.as_bytes())?;
        w.queue(Clear(ClearType::UntilNewLine))?;
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
            if line_idx == data.focus().ln {
                Self::DELIM_BOLD
            } else {
                Self::DELIM
            }
            .as_bytes(),
        )?;

        Ok(())
    }
}

pub struct ClassicFooter;

impl<W: Write> Footer<W> for ClassicFooter {
    fn rows(&self) -> usize {
        1
    }

    fn draw(&mut self, w: &mut W, data: &RenderData) -> Result<()> {
        write!(
            w,
            "      ╰─── Lines: {} ─── Chars: {} ─── Ln: {}, Col: {}",
            data.line_count(),
            data.char_count(),
            data.focus().ln,
            data.focus().col.min(data.current_line().len())
        )?;
        w.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }
}
