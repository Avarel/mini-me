use std::io::Write;

use super::data::RenderData;
use crossterm::{QueueableCommand, Result, terminal::{Clear, ClearType}};

pub trait Footer<W> {
    fn height(&self) -> usize;
    fn draw(&mut self, write: &mut W, data: &RenderData) -> Result<()>;
}

pub struct NoFooter;

impl<W> Footer<W> for NoFooter {
    fn height(&self) -> usize {
        0
    }

    fn draw(&mut self, _: &mut W, _: &RenderData) -> Result<()> {
        Ok(())
    }
}

pub struct ClassicFooter;

impl<W: Write> Footer<W> for ClassicFooter {
    fn height(&self) -> usize {
        1
    }

    fn draw(&mut self, w: &mut W, data: &RenderData) -> Result<()> {
        w.write(b"\n")?;
        write!(
            w,
            "      ╰──┤ Lines: {} ├─┤ Chars: {} ├─┤ Ln: {}, Col: {}",
            data.line_count(),
            data.char_count(),
            data.cursor().ln,
            data.cursor().col.min(data.current_line().len())
        )?;
        w.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }
}