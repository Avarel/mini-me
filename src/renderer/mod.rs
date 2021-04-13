use std::{
    borrow::Cow,
    io::{self, Write},
};

use crate::util::{trimmed, Cursor};

/// Full renderer.
pub mod full;

/// Preset styles.
pub mod styles;

use crate::Result;

pub trait Renderer {
    fn draw(&mut self, data: RenderData) -> Result<()>;
    fn clear_draw(&mut self) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
    fn finish(self) -> Result<()>;
}

use crossterm::{
    style::{Color, ResetColor, SetBackgroundColor},
    QueueableCommand,
};
use ropey::{Rope, RopeSlice};
#[derive(Clone, Copy)]
pub struct RenderData<'b> {
    buf: &'b Rope,
    focus: Cursor,
    anchor: Option<Cursor>,
}

impl<'b> RenderData<'b> {
    pub fn new(buf: &'b Rope, focus: Cursor, anchor: Option<Cursor>) -> Self {
        Self { buf, focus, anchor }
    }

    pub fn focus(&self) -> Cursor {
        self.focus
    }

    pub fn line_count(&self) -> usize {
        self.buf.len_lines()
    }

    pub fn char_count(&self) -> usize {
        self.buf.len_chars()
    }

    fn write_rope(write: &mut dyn Write, rope: RopeSlice<'_>) -> io::Result<()> {
        rope.chunks()
            .map(|c| c.as_bytes())
            .try_for_each(|c| write.write_all(c))
    }

    pub fn write_line(&self, line_idx: usize, write: &mut dyn Write) -> Result<()> {
        if let Some(anchor) = self.anchor {
            let (mut start, mut end) = (self.focus.min(anchor), self.focus.max(anchor));
            let line = trimmed(self.buf.line(line_idx));
            if start.ln < line_idx && line_idx < end.ln {
                write.queue(SetBackgroundColor(Color::DarkGrey))?;
                Self::write_rope(write, line)?;
                write.queue(ResetColor)?;
                return Ok(());
            } else if start.ln == end.ln && line_idx == start.ln {
                Self::write_rope(write, line.slice(..start.col))?;
                write.queue(SetBackgroundColor(Color::DarkGrey))?;
                Self::write_rope(write, line.slice(start.col..end.col))?;
                write.queue(ResetColor)?;
                Self::write_rope(write, line.slice(end.col..))?;

                write.queue(ResetColor)?;
                return Ok(());
            } else if line_idx == start.ln {
                start.col = start.col.clamp(0, line.len_chars());
                Self::write_rope(write, line.slice(..start.col))?;
                write.queue(SetBackgroundColor(Color::DarkGrey))?;
                Self::write_rope(write, line.slice(start.col..))?;
                write.queue(ResetColor)?;
                return Ok(());
            } else if line_idx == end.ln {
                end.col = end.col.clamp(0, line.len_chars());
                write.queue(SetBackgroundColor(Color::DarkGrey))?;
                Self::write_rope(write, line.slice(..end.col))?;
                write.queue(ResetColor)?;
                Self::write_rope(write, line.slice(end.col..))?;
                return Ok(());
            }
        }
        trimmed(self.buf.line(line_idx))
            .chunks()
            .map(|c| c.as_bytes())
            .try_for_each(|c| write.write_all(c))?;
        Ok(())
    }

    pub fn line(&self, index: usize) -> Cow<str> {
        trimmed(self.buf.line(index)).into()
    }

    pub fn last_line(&self) -> Cow<str> {
        self.line(self.buf.len_lines() - 1)
    }

    pub fn current_line(&self) -> Cow<str> {
        self.line(self.focus.ln)
    }
}
