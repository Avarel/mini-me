use std::{
    borrow::Cow,
    io::{self, Write},
};

use crate::util::{Cursor, RopeExt};

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
        self.buf
            .line_trimmed(line_idx)
            .chunks()
            .map(|c| c.as_bytes())
            .try_for_each(|c| write.write_all(c))
    }

    pub fn line(&self, index: usize) -> Cow<str> {
        self.buf.line_trimmed(index).into()
    }

    pub fn last_line(&self) -> Cow<str> {
        self.line(self.buf.len_lines() - 1)
    }

    pub fn current_line(&self) -> Cow<str> {
        self.line(self.cursor.ln)
    }
}
