use std::io::Write;

use super::Editor;
use crate::Result;

pub mod classic;
pub mod fancy;

pub trait Style<W> {
    fn footer_rows(&self) -> usize;
    fn header_rows(&self) -> usize;
    fn gutter_width(&self) -> usize;
    fn draw_header(&mut self, write: &mut W, data: &Editor) -> Result<()>;
    fn draw_gutter(&mut self, write: &mut W, line_idx: usize, data: &Editor) -> Result<()>;
    fn draw_footer(&mut self, write: &mut W, data: &Editor) -> Result<()>;
}

pub struct NoStyle;

impl<W> Style<W> for NoStyle {
    fn footer_rows(&self) -> usize {
        0
    }

    fn header_rows(&self) -> usize {
        0
    }

    fn gutter_width(&self) -> usize {
        0
    }

    fn draw_header(&mut self, write: &mut W, data: &Editor) -> Result<()> {
        Ok(())
    }

    fn draw_gutter(&mut self, write: &mut W, line_idx: usize, data: &Editor) -> Result<()> {
        Ok(())
    }

    fn draw_footer(&mut self, write: &mut W, data: &Editor) -> Result<()> {
        Ok(())
    }
}