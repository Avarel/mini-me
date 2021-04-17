use std::io::Write;

use super::{Footer, Header, Margin};
use crate::{renderer::Editor, Result};
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

    fn draw(&mut self, w: &mut W, _: &Editor) -> Result<()> {
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

    fn draw(&mut self, write: &mut W, line_idx: usize, data: &Editor) -> Result<()> {
        if line_idx + 1 > data.line_count() {
            write!(write, "{:>width$}", "", width = Self::WIDTH)?;
        } else {
            write!(write, "{:>width$}", line_idx + 1, width = Self::WIDTH)?;
        }

        write.write(
            if line_idx == data.selection.focus.ln {
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

    fn draw(&mut self, w: &mut W, data: &Editor) -> Result<()> {
        write!(
            w,
            "      ╰─── Lines: {} ─── Chars: {} ─── Ln: {}, Col: {}",
            data.line_count(),
            data.char_count(),
            data.selection.focus.ln,
            data.selection.focus.col.min(data.curr_ln_len())
        )?;
        w.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }
}
