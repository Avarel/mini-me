use crate::{Cursor, RenderData};

use std::convert::TryInto;
use std::io::{stdout, Write};

use crossterm::{
    cursor::*,
    terminal::{Clear, ClearType},
    queue, Result,
};

use super::Renderer;

pub struct FullRenderer<'b> {
    write: &'b mut dyn Write,
    #[doc(hidden)]
    pub(super) pds: PreviousDrawState,
    /// Function to draw the prompt.
    formatter: Option<&'b dyn Fn(usize, &RenderData) -> String>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PreviousDrawState {
    pub height: usize,
    pub cursor: Cursor,
}

impl Renderer for FullRenderer<'_> {
    /// Draw the prompt.
    fn draw(&mut self, term: &RenderData) -> Result<()> {
        // Handle empty buffer.
        if term.buffers.is_empty() {
            if let Some(f) = &self.formatter {
                let string = &f(0, term);
                self.write_str(string)?;
            }
            self.pds.height = 1;
            return self.flush();
        }

        // Print out the contents.
        for i in 0..term.buffers.len() {
            self.draw_line(term, i)?;
            if i < term.buffers.len() - 1 {
                // The last line should not have any new-line attached to it.
                self.new_line()?;
            }
        }

        queue!(self.write, Clear(ClearType::FromCursorDown))?;

        self.pds.height = term.buffers.len();
        self.pds.cursor.line = term.buffers.len() - 1;
        self.pds.cursor.index = term.buffers.last().unwrap().len();

        self.draw_cursor(term)?;
        self.flush()
    }

    /// Clear the drawn prompt on the screen.
    fn clear_draw(&mut self) -> Result<()> {
        self.move_cursor_to_bottom()?;
        self.clear_line()?;

        self.move_cursor_up(self.pds.height - 1)?;
        queue!(self.write, Clear(ClearType::FromCursorDown))?;

        self.pds.height = 0;
        self.pds.cursor.line = 0;
        self.pds.cursor.index = 0;

        Ok(())
    }

    /// Clear the line on the current cursor.
    fn clear_line(&mut self) -> Result<()> {
        queue!(self.write, Clear(ClearType::CurrentLine))?;
        self.cursor_to_lmargin()?;
        
        self.pds.cursor.index = 0;

        Ok(())
    }

    /// Redraw the screen.
    fn redraw(&mut self, term: &RenderData) -> Result<()> {
        queue!(self.write, Hide)?;
        self.move_cursor_up(self.pds.cursor.line)?;
        self.draw(term)?;
        queue!(self.write, Show)?;

        self.flush()
    }

    fn flush(&mut self) -> Result<()> {
        self.write.flush()?;
        Ok(())
    }
}

impl<'w> FullRenderer<'w> {
    pub fn render_to(write: &'w mut dyn Write) -> Self {
        FullRenderer {
            write,
            ..Default::default()
        }
    }

    pub fn render_with_formatter<F: Fn(usize, &RenderData) -> String>(f: &'w F) -> Self {
        FullRenderer {
            formatter: Some(f),
            ..Default::default()
        }
    }

    pub fn render_with_formatter_to<F: Fn(usize, &RenderData) -> String>(write: &'w mut dyn Write, f: &'w F) -> Self {
        FullRenderer {
            write,
            formatter: Some(f),
            ..Default::default()
        }
    }

    fn write_str(&mut self, s: &str) -> Result<()> {
        write!(self.write, "{}", s)?;
        Ok(())
    }

    
    fn write_line(&mut self, s: &str) -> Result<()> {
        write!(self.write, "{}\n", s)?;
        Ok(())
    }

    // Position the cursor.
    // At this point the cursor is pointed at the very end of the last line.
    pub fn draw_cursor(&mut self, term: &RenderData) -> Result<()> {
        self.move_cursor_to_line(term.cursor.line)?;
        self.move_cursor_to_index(term.cursor.index.min(term.current_line_len()))
    }

    /// Draw the line given an index.
    /// This method does not move the cursor.
    pub fn draw_line(&mut self, term: &RenderData, line: usize) -> Result<()> {
        self.cursor_to_lmargin()?;

        if let Some(f) = &self.formatter {
            let string = &f(line, term);
            self.write_str(string)?;
        } else {
            self.write_str(term.line(line))?;
        }

        // self.write_str(&term.buffers[line])?;
        queue!(self.write, Clear(ClearType::UntilNewLine))?;
        Ok(())
    }

    /// Insert a new line on the screen.
    #[inline]
    pub fn new_line(&mut self) -> Result<()> {
        self.write_line("")
    }

    /// Move the current cursor to the last line.
    #[inline]
    pub fn move_cursor_to_bottom(&mut self) -> Result<()> {
        self.move_cursor_down(self.pds.height - self.pds.cursor.line - 1)
    }

    pub fn move_cursor_to_line(&mut self, line: usize) -> Result<()> {
        let pds_line = self.pds.cursor.line;

        if pds_line > line {
            self.move_cursor_up(pds_line - line)
        } else if pds_line < line {
            self.move_cursor_down(line - pds_line)
        } else {
            Ok(())
        }
    }

    pub fn move_cursor_to_index(&mut self, index: usize) -> Result<()> {
        let pds_index = self.pds.cursor.index;

        if index < pds_index {
            self.move_cursor_left(pds_index - index)
        } else if index > pds_index {
            self.move_cursor_right(index - pds_index)
        } else {
            Ok(())
        }
    }

    /// Move the cursor to the end of the current line.
    /// This method is not safe to use if the cursor is not at `line:index`,
    #[inline]
    pub fn move_cursor_to_end(&mut self, term: &RenderData) -> Result<()> {
        let pds = self.pds;
        let len = term.current_line_len();
        if pds.cursor.index > len {
            self.move_cursor_left(pds.cursor.index - len)
        } else if pds.cursor.index < len {
            self.move_cursor_right(len - pds.cursor.index)
        } else {
            Ok(())
        }
    }

    /// Move the cursor to the beginning of the line.
    #[inline]
    pub fn move_cursor_to_start(&mut self, term: &RenderData) -> Result<()> {
        self.move_cursor_left(term.cursor.index)?;
        Ok(())
    }

    /// Move the curser to the terminal left margin.
    #[doc(hidden)]
    fn cursor_to_lmargin(&mut self) -> Result<()> {
        if let Ok((_, r)) = crossterm::cursor::position() {
            queue!(self.write, MoveTo(0, r))?;
        } else {
            // Fallback
            queue!(self.write, MoveLeft(std::u16::MAX))?;
        }
        Ok(())
    }

    /// Move the cursor one line up.
    #[inline]
    pub fn move_cursor_up(&mut self, n: usize) -> Result<()> {
        if n != 0 {
            queue!(self.write, MoveUp(n.try_into().unwrap_or(std::u16::MAX)))?;
            self.pds.cursor.line -= n;
        }
        Ok(())
    }

    /// Move the cursor one line down.
    #[inline]
    pub fn move_cursor_down(&mut self, n: usize) -> Result<()> {
        if n != 0 {
            queue!(self.write, MoveDown(n.try_into().unwrap_or(std::u16::MAX)))?;
            self.pds.cursor.line += n;
        }
        Ok(())
    }

    /// Move the cursor leftward using nondestructive backspaces.
    #[inline]
    pub fn move_cursor_left(&mut self, n: usize) -> Result<()> {
        if n != 0 {
            queue!(self.write, MoveLeft(n.try_into().unwrap_or(std::u16::MAX)))?;
            self.pds.cursor.index -= n;
        }
        Ok(())
    }

    /// Move the cursor rightward.
    #[inline]
    pub fn move_cursor_right(&mut self, n: usize) -> Result<()> {
        if n != 0 {
            queue!(self.write, MoveRight(n.try_into().unwrap_or(std::u16::MAX)))?;
            self.pds.cursor.index += n;
        }
        Ok(())
    }
}

impl Default for FullRenderer<'_> {
    fn default() -> Self {
        // Since stdout() lives for the entirety of the process.
        // This is safe since the handle will always be valid.
        // The only time where it may die is during shutdown.
        let out = Box::new(stdout());
        FullRenderer {
            write: Box::leak(out),
            pds: PreviousDrawState::default(),
            formatter: None,
        }
    }
}