use std::io;
use std::cell::RefCell;
use std::cell::RefMut;
use std::cell::Ref;
use crate::{MultilineTerm, Cursor};

pub struct Renderer {
    /// The mode of anchoring for the multiline terminal.
    pub anchor: AnchorMode,
    pub mode: RenderMode,
    /// Helper field for anchor mode `Bottom`.
    pub empty_padding: usize,
    pub previous_draw: RefCell<PreviousDrawState>,
}

pub struct PreviousDrawState {
    pub height: usize,
    pub cursor: Cursor 
}

impl Default for PreviousDrawState {
    fn default() -> Self {
        Self {
            height: 0,
            cursor: Cursor { line: 0, index: 0 },
        }
    }
}

/// The mode of anchoring of the multiline prompt.
#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum AnchorMode {
    /// Default mode of anchoring for the multiline prompt.
    /// The multiline prompt will always be anchored where it is first printed.
    InPlace,
    /// Alternative mode of anchoring for the multiline prompt.
    /// The multiline prompt will always be anchored at the bottom of the terminal.
    Bottom,
}

impl Default for AnchorMode {
    fn default() -> Self {
        AnchorMode::InPlace
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderMode {
    /// Allows for proper prompt drawing everytime and
    /// terminal-wide visual updates to the input.
    Full,
    /// Most performant, but may be limited when trying
    /// to render more advanced options.
    Lazy,
}

impl Default for RenderMode {
    fn default() -> Self {
        RenderMode::Lazy
    }
}

impl Renderer {
    fn previous_draw(&self) -> Ref<PreviousDrawState> {
        self.previous_draw.borrow()
    }

    fn previous_draw_mut(&self) -> RefMut<PreviousDrawState> {
        self.previous_draw.borrow_mut()
    }

    /// Draw the prompt.
    pub fn draw(&self, term: &MultilineTerm) -> io::Result<()> {
        // Handle empty buffer.
        if term.buffers.is_empty() {
            if let Some(f) = &term.options.gutter {
                term.inner.write_str(&f(0, term))?;
            }
            return Ok(())
        }

        if let AnchorMode::Bottom = self.anchor {
            // Print out the padding.
            for _ in 0..self.empty_padding {
                self.new_line(term)?;
            }
        }

        // Print out the contents.
        for i in 0..term.buffers.len() {
            self.draw_line(term, i)?;
            if i < term.buffers.len() - 1 {
                // The last line should not have any new-line attached to it.
                self.new_line(term)?;
            }
        }


        let last_len = term.buffers.last().unwrap().len();
        {
            *self.previous_draw.borrow_mut() = PreviousDrawState {
                height: self.empty_padding + term.buffers.len(),
                cursor: Cursor { line: term.buffers.len() - 1, index: last_len }
            }.into();
        }

        // Position the cursor.
        // At this point the cursor is pointed at the very end of the last line.
        self.move_cursor_up(term, term.buffers.len() - term.cursor.line - 1)?;
        if term.cursor.index < last_len {
            self.move_cursor_left(term, last_len - term.cursor.index)?;
        } else if term.cursor.index > last_len && term.cursor.line + 1 != term.buffers.len() {
            self.move_cursor_right(term, term.current_line_len().min(term.cursor.index) - last_len)?;
        }

        Ok(())
    }

    /// Clear the drawn prompt on the screen.
    pub fn clear_draw(&self, term: &MultilineTerm) -> io::Result<()> {
        self.move_cursor_to_bottom(term)?;
        term.inner.clear_line()?;
        if term.buffers.len() != 0 {
            term.inner.clear_last_lines(term.buffers.len() - 1 + self.empty_padding)?;
        }
        Ok(())
    }

    /// Redraw the screen.
    pub fn redraw(&self, term: &MultilineTerm, hint: RenderMode) -> io::Result<()> {
        if self.mode == RenderMode::Lazy && hint == RenderMode::Lazy {
            self.clear_current_line(term)?;
            self.draw_current_line(term)
        } else {
            let pdh = { self.previous_draw().height };
            let pdl = { self.previous_draw().cursor.line };
            self.move_cursor_down(term, pdh - pdl)?;
            term.inner.clear_line()?;
            if pdh != 0 {
                dbg!(pdh);
                term.inner.clear_last_lines(pdh + self.empty_padding)?;
            }

            // self.draw(term)
            Ok(())
        }
    }

    /// Draw the line given an index.
    /// This method does not move the cursor.
    pub fn draw_line(&self, term: &MultilineTerm, index: usize) -> io::Result<()> {
        if let Some(f) = &term.options.gutter {
            term.inner.write_str(&f(index, term))?;
        }
        term.inner.write_str(&term.buffers[index])
    }

    /// Draw the current line and move the cursor appropriately.
    pub fn draw_current_line(&self, term: &MultilineTerm) -> io::Result<()> {
        self.draw_line(term, term.cursor.line)?;
        self.previous_draw_mut().cursor.index = term.buffers[term.cursor.line].len();
        // disable this check if you want overflow cursor
        if term.cursor.index < term.current_line_len() {
            self.move_cursor_left(term, term.current_line_len() - term.cursor.index)?;
        }
        Ok(())
    }

    /// Clear the current line on the screen.
    #[inline]
    pub fn clear_current_line(&self, term: &MultilineTerm) -> io::Result<()> {
        term.inner.clear_line()
    }

    /// Insert a new line on the screen.
    #[inline]
    pub fn new_line(&self, term: &MultilineTerm) -> io::Result<()> {
        term.inner.write_line("")
    }

        /// Move the current cursor to the last line.
    #[inline]
    pub fn move_cursor_to_bottom(&self, term: &MultilineTerm) -> io::Result<usize> {
        if term.buffers.len() == 0 { return Ok(0) }
        self.move_cursor_down(term, term.buffers.len() - term.cursor.line - 1)?;
        Ok(term.buffers.len())
    }

    /// Move the cursor to the end of the current line.
    /// This method is not safe to use if the cursor is not at `line:index`,
    #[inline]
    pub fn move_cursor_to_end(&self, term: &MultilineTerm) -> io::Result<usize> {
        let len = term.current_line().len();
        if term.cursor.index > len {
            self.move_cursor_left(term, term.cursor.index - len)?;
        } else if term.cursor.index < len {
            self.move_cursor_right(term, len - term.cursor.index)?;
        }
        Ok(len)
    }

    /// Move the cursor to the beginning of the line.
    #[inline]
    pub fn move_cursor_to_start(&self, term: &MultilineTerm) -> io::Result<usize> {
        self.move_cursor_left(term, term.cursor.index)?;
        Ok(0)
    }

    /// Move the cursor one line up.
    #[inline]
    pub fn move_cursor_up(&self, term: &MultilineTerm, n: usize) -> io::Result<usize> {
        term.inner.move_cursor_up(n)?;
        self.previous_draw_mut().cursor.line -= n;
        if term.cursor.line == 0 {
            Ok(0)
        } else {
            Ok(term.cursor.line - n)
        }
    }

    /// Move the cursor one line down.
    #[inline]
    pub fn move_cursor_down(&self, term: &MultilineTerm, n: usize) -> io::Result<usize> {
        term.inner.move_cursor_down(n)?;
        self.previous_draw_mut().cursor.line += n;
        Ok(term.cursor.line + n)
    }

    /// Move the cursor leftward using nondestructive backspaces.
    #[inline]
    pub fn move_cursor_left(&self, term: &MultilineTerm, n: usize) -> io::Result<usize> {
        term.inner.move_cursor_left(n)?;
        self.previous_draw_mut().cursor.index -= n;
        if term.cursor.index == 0 {
            Ok(0)
        } else {
            Ok(term.cursor.index - n)
        }
    }

    /// Move the cursor rightward.
    /// This method is not safe to use if the cursor is not at `line:index`,
    /// as it draws from the buffer to move forward.
    #[inline]
    pub fn move_cursor_right(&self, term: &MultilineTerm, n: usize) -> io::Result<usize> {
        term.inner.move_cursor_right(n)?;
        self.previous_draw_mut().cursor.index += n;
        Ok(term.cursor.index + n)
    }
}