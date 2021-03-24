use std::{
    borrow::Cow,
    io::{self, Write},
};

use crate::util::{trimmed, Cursor};

pub mod full;
pub mod styles;

use crate::Result;

pub trait Renderer {
    fn draw(&mut self, data: RenderData) -> Result<()>;
    fn redraw(&mut self, data: RenderData) -> Result<()>;
    fn clear_draw(&mut self) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
    fn finish(self) -> Result<()>;
}

use ropey::Rope;
#[derive(Clone, Copy)]
pub struct RenderData<'b> {
    buf: &'b Rope,
    cursor: &'b Cursor,
}

impl<'b> RenderData<'b> {
    pub fn new(buf: &'b Rope, cursor: &'b Cursor) -> Self {
        Self { buf, cursor }
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn line_count(&self) -> usize {
        self.buf.len_lines()
    }

    pub fn char_count(&self) -> usize {
        self.buf.len_chars()
    }

    pub fn write_line(&self, line_idx: usize, write: &mut dyn Write) -> io::Result<()> {
        trimmed(self.buf.line(line_idx))
            .chunks()
            .map(|c| c.as_bytes())
            .try_for_each(|c| write.write_all(c))
    }

    pub fn line(&self, index: usize) -> Cow<str> {
        trimmed(self.buf.line(index)).into()
    }

    pub fn last_line(&self) -> Cow<str> {
        self.line(self.buf.len_lines() - 1)
    }

    pub fn current_line(&self) -> Cow<str> {
        self.line(self.cursor.ln)
    }
}
