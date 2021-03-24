use std::io::Write;

use super::RenderData;
use crate::Result;

pub mod classic;
pub mod fancy;

pub trait Header<W> {
    fn rows(&self) -> usize;
    fn draw(&mut self, write: &mut W, data: &RenderData) -> Result<()>;
}

pub struct NoHeader;

impl<W> Header<W> for NoHeader {
    fn rows(&self) -> usize {
        0
    }

    fn draw(&mut self, _: &mut W, _: &RenderData) -> Result<()> {
        Ok(())
    }
}

impl<W: Write> Header<W> for Box<dyn Header<W>> {
    fn rows(&self) -> usize {
        (**self).rows()
    }
    fn draw(&mut self, write: &mut W, data: &RenderData) -> Result<()> {
        (**self).draw(write, data)
    }
}

pub trait Margin<W> {
    fn width(&self) -> usize;
    fn draw(&mut self, write: &mut W, line_idx: usize, data: &RenderData) -> Result<()>;
}

impl<W: Write> Margin<W> for Box<dyn Margin<W>> {
    fn width(&self) -> usize {
        (**self).width()
    }
    fn draw(&mut self, write: &mut W, line_idx: usize, data: &RenderData) -> Result<()> {
        (**self).draw(write, line_idx, data)
    }
}

pub struct NoMargin;

impl<W> Margin<W> for NoMargin {
    fn width(&self) -> usize {
        0
    }

    fn draw(&mut self, _: &mut W, _: usize, _: &RenderData) -> Result<()> {
        Ok(())
    }
}

pub trait Footer<W> {
    fn rows(&self) -> usize;
    fn draw(&mut self, write: &mut W, data: &RenderData) -> Result<()>;
}

impl<W: Write> Footer<W> for Box<dyn Footer<W>> {
    fn rows(&self) -> usize {
        (**self).rows()
    }
    fn draw(&mut self, write: &mut W, data: &RenderData) -> Result<()> {
        (**self).draw(write, data)
    }
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
