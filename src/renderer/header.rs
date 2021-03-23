use std::io::Write;

use super::data::RenderData;
use crossterm::{
    terminal::{Clear, ClearType},
    QueueableCommand, Result,
};

pub trait Header<W> {
    fn height(&self) -> usize;
    fn draw(&mut self, write: &mut W, data: &RenderData) -> Result<()>;
}

pub struct NoHeader;

impl<W> Header<W> for NoHeader {
    fn height(&self) -> usize {
        0
    }

    fn draw(&mut self, _: &mut W, _: &RenderData) -> Result<()> {
        Ok(())
    }
}

pub struct ClassicHeader;

impl<W: Write> Header<W> for ClassicHeader {
    fn height(&self) -> usize {
        1
    }

    fn draw(&mut self, w: &mut W, _: &RenderData) -> Result<()> {
        write!(w, "      ╭─── Input Prompt ─────────")?;
        w.queue(Clear(ClearType::UntilNewLine))?;
        w.write(b"\n")?;
        Ok(())
    }
}
