use std::io::Write;

use super::data::RenderData;
use crate::Result;
use crossterm::{
    terminal::{Clear, ClearType},
    QueueableCommand,
};

pub trait Footer<W> {
    fn rows(&self) -> usize;
    fn draw(&mut self, write: &mut W, data: &RenderData) -> Result<()>;
}

pub struct NoFooter;

impl<W> Footer<W> for NoFooter {
    fn rows(&self) -> usize {
        0
    }

    fn draw(&mut self, _: &mut W, _: &RenderData) -> Result<()> {
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
