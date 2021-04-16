use std::io::{self, Write};

use crate::{editor::Editor, util::trimmed};

/// Full renderer.
pub mod full;

/// Preset styles.
pub mod styles;

use crate::Result;

pub trait Renderer {
    fn draw(&mut self, data: &Editor) -> Result<()>;
    fn clear_draw(&mut self) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
    fn finish(self) -> Result<()>;
}

use crossterm::{
    style::{Color, ResetColor, SetBackgroundColor},
    QueueableCommand,
};
use ropey::RopeSlice;

impl Editor {
    fn write_rope(write: &mut dyn Write, rope: RopeSlice<'_>) -> io::Result<()> {
        rope.chunks()
            .map(|c| c.as_bytes())
            .try_for_each(|c| write.write_all(c))
    }

    pub fn write_line(&self, line_idx: usize, write: &mut dyn Write) -> Result<()> {
        if let Some(anchor) = self.selection.anchor {
            let (mut start, mut end) = (
                self.selection.focus.min(anchor),
                self.selection.focus.max(anchor),
            );
            let line = trimmed(self.buf.line(line_idx));
            if start.ln < line_idx && line_idx < end.ln {
                write.queue(SetBackgroundColor(Color::DarkGrey))?;
                Self::write_rope(write, line)?;
                write.queue(ResetColor)?;
                return Ok(());
            } else if start.ln == end.ln && line_idx == start.ln {
                Self::write_rope(write, line.slice(..start.col))?;
                write.queue(SetBackgroundColor(Color::DarkGrey))?;
                write!(write, "{}", crossterm::style::Attribute::Bold)?;
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
}
