use crate::{Cursor, MultilineTerm};

use std::cell::Cell;
use std::convert::TryInto;
use std::io::{stdout, Write};

use crossterm::{
    cursor::*,
    terminal::{Clear, ClearType},
    Output, QueueableCommand, Result,
};

use super::Renderer;

#[derive(Default)]
pub struct FullRenderer {
    #[doc(hidden)]
    pds: Cell<PreviousDrawState>,
    /// Function to draw the prompt.
    gutter: Option<Box<dyn Fn(usize, &MultilineTerm) -> String>>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreviousDrawState {
    pub height: usize,
    pub cursor: Cursor,
}

impl Renderer for FullRenderer {
    /// Draw the prompt.
    fn draw(&self, term: &MultilineTerm) -> Result<()> {
        // Handle empty buffer.
        if term.buffers.is_empty() {
            if let Some(f) = &self.gutter {
                self.write_str(&f(0, term))?;
            }
            self.update_pds(|pds| pds.height = 1);
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

        stdout().queue(Clear(ClearType::FromCursorDown))?;

        self.update_pds(|pds| {
            pds.height = term.buffers.len();
            pds.cursor.line = term.buffers.len() - 1;
            pds.cursor.index = term.buffers.last().unwrap().len();
        });

        self.draw_cursor(term)?;
        self.flush()
    }

    /// Clear the drawn prompt on the screen.
    fn clear_draw(&self) -> Result<()> {
        self.move_cursor_to_bottom()?;
        self.clear_line()?;

        self.move_cursor_up(self.pds().height - 1)?;
        stdout().queue(Clear(ClearType::FromCursorDown))?;

        self.update_pds(|pds| {
            pds.height = 0;
            pds.cursor.line = 0;
            pds.cursor.index = 0;
        });

        Ok(())
    }

    /// Clear the line on the current cursor.
    fn clear_line(&self) -> Result<()> {
        stdout().queue(Clear(ClearType::CurrentLine))?;
        self.cursor_to_lmargin()?;
        self.update_pds(|pds| {
            pds.cursor.index = 0;
        });
        Ok(())
    }

    /// Redraw the screen.
    fn redraw(&self, term: &MultilineTerm) -> Result<()> {
        stdout().queue(Hide)?;
        self.move_cursor_up(self.pds().cursor.line)?;
        self.draw(term)?;
        stdout().queue(Show)?;

        self.flush()
    }

    fn flush(&self) -> Result<()> {
        stdout().flush()?;
        Ok(())
    }
}

impl FullRenderer {
    pub fn with_gutter<F: 'static + Fn(usize, &MultilineTerm) -> String>(f: F) -> Self {
        FullRenderer {
            pds: Cell::new(PreviousDrawState::default()),
            gutter: Some(Box::new(f)),
        }
    }

    #[doc(hidden)]
    pub(super) fn update_pds<F: FnOnce(&mut PreviousDrawState)>(&self, f: F) {
        let mut pds = self.pds();
        f(&mut pds);
        self.pds.set(pds);
    }

    #[doc(hidden)]
    pub(super) fn pds(&self) -> PreviousDrawState {
        self.pds.get()
    }

    fn write_str(&self, s: &str) -> Result<()> {
        stdout().queue(Output(s))?;
        Ok(())
    }

    fn write_line(&self, s: &str) -> Result<()> {
        stdout().queue(Output(s))?.queue(Output('\n'))?;
        Ok(())
    }

    // Position the cursor.
    // At this point the cursor is pointed at the very end of the last line.
    pub fn draw_cursor(&self, term: &MultilineTerm) -> Result<()> {
        self.move_cursor_to_line(term.cursor.line)?;
        self.move_cursor_to_index(term.cursor.index.min(term.current_line_len()))
    }

    /// Draw the line given an index.
    /// This method does not move the cursor.
    pub fn draw_line(&self, term: &MultilineTerm, line: usize) -> Result<()> {
        self.cursor_to_lmargin()?;
        if let Some(f) = &self.gutter {
            self.write_str(&f(line, term))?;
        }
        self.write_str(&term.buffers[line])?;
        stdout().queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }

    /// Insert a new line on the screen.
    #[inline]
    pub fn new_line(&self) -> Result<()> {
        self.write_line("")
    }

    /// Move the current cursor to the last line.
    #[inline]
    pub fn move_cursor_to_bottom(&self) -> Result<()> {
        self.move_cursor_down(self.pds().height - self.pds().cursor.line - 1)
    }

    pub fn move_cursor_to_line(&self, line: usize) -> Result<()> {
        let pds_line = self.pds().cursor.line;

        if pds_line > line {
            self.move_cursor_up(pds_line - line)
        } else if pds_line < line {
            self.move_cursor_down(line - pds_line)
        } else {
            Ok(())
        }
    }

    pub fn move_cursor_to_index(&self, index: usize) -> Result<()> {
        let pds_index = self.pds().cursor.index;

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
    pub fn move_cursor_to_end(&self, term: &MultilineTerm) -> Result<()> {
        let pds = self.pds();
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
    pub fn move_cursor_to_start(&self, term: &MultilineTerm) -> Result<()> {
        self.move_cursor_left(term.cursor.index)?;
        Ok(())
    }

    /// Move the curser to the terminal left margin.
    #[doc(hidden)]
    fn cursor_to_lmargin(&self) -> Result<()> {
        if let Ok((_, r)) = crossterm::cursor::position() {
            stdout().queue(MoveTo(0, r))?;
        } else {
            // Fallback
            stdout().queue(MoveLeft(std::u16::MAX))?;
        }
        Ok(())
    }

    /// Move the cursor one line up.
    #[inline]
    pub fn move_cursor_up(&self, n: usize) -> Result<()> {
        if n != 0 {
            stdout().queue(MoveUp(n.try_into().unwrap_or(std::u16::MAX)))?;
            self.update_pds(|pds| pds.cursor.line -= n);
        }
        Ok(())
    }

    /// Move the cursor one line down.
    #[inline]
    pub fn move_cursor_down(&self, n: usize) -> Result<()> {
        if n != 0 {
            stdout().queue(MoveDown(n.try_into().unwrap_or(std::u16::MAX)))?;
            self.update_pds(|pds| pds.cursor.line += n);
        }
        Ok(())
    }

    /// Move the cursor leftward using nondestructive backspaces.
    #[inline]
    pub fn move_cursor_left(&self, n: usize) -> Result<()> {
        if n != 0 {
            stdout().queue(MoveLeft(n.try_into().unwrap_or(std::u16::MAX)))?;
            self.update_pds(|pds| pds.cursor.index -= n);
        }
        Ok(())
    }

    /// Move the cursor rightward.
    #[inline]
    pub fn move_cursor_right(&self, n: usize) -> Result<()> {
        if n != 0 {
            stdout().queue(MoveRight(n.try_into().unwrap_or(std::u16::MAX)))?;
            self.update_pds(|pds| pds.cursor.index += n);
        }
        Ok(())
    }
}
