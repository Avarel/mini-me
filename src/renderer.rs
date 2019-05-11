use std::io;
use std::cell::Cell;
use crate::{MultilineTerm, Cursor};

pub struct Renderer {
    pub mode: RenderMode,
    /// Helper field for anchor mode `Bottom`.
    pub empty_padding: usize,
    pub previous_draw: Cell<PreviousDrawState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub fn new() -> Self {
        Self {
            mode: RenderMode::Lazy,
            empty_padding: 0,
            previous_draw: std::cell::Cell::new(Default::default()),
        }
    }

    #[doc(hidden)]
    fn update_pds<F: FnOnce(&mut PreviousDrawState)>(&self, f: F) {
        let mut pds = self.pds();
        f(&mut pds);
        self.previous_draw.set(pds);
    }

    #[doc(hidden)]
    fn pds(&self) -> PreviousDrawState {
        self.previous_draw.get()
    }

    /// Draw the prompt.
    pub fn draw(&self, term: &MultilineTerm) -> io::Result<()> {
        // Handle empty buffer.
        if term.buffers.is_empty() {
            if let Some(f) = &term.gutter {
                term.inner.write_str(&f(0, term))?;
            }
            self.update_pds(|pds| pds.height = 1);
            return Ok(())
        }

        // Print out the contents.
        for i in 0..term.buffers.len() {
            self.draw_line(term, i)?;
            if i < term.buffers.len() - 1 {
                // The last line should not have any new-line attached to it.
                self.new_line(term)?;
            }
        }

        self.update_pds(|pds| {
            pds.height = self.empty_padding + term.buffers.len();
            pds.cursor.line = term.buffers.len() - 1;
            pds.cursor.index = term.buffers.last().unwrap().len();
        });

        self.draw_cursor(term)
    }

    // Position the cursor.
    // At this point the cursor is pointed at the very end of the last line.
    pub fn draw_cursor(&self, term: &MultilineTerm) -> io::Result<()> {
        let pds_index = self.pds().cursor.index;
        self.move_cursor_up(term, term.buffers.len() - term.cursor.line - 1)?;
        
        if term.cursor.index > term.current_line_len() {
            self.move_cursor_left(term, pds_index - term.current_line_len())
        } else if term.cursor.index < pds_index {
            self.move_cursor_left(term, pds_index - term.cursor.index)
        } else if term.cursor.index > pds_index {
            self.move_cursor_right(term, term.cursor.index - pds_index)
        } else {
            Ok(())
        }
    }

    /// Clear the drawn prompt on the screen.
    pub fn clear_draw(&self, term: &MultilineTerm) -> io::Result<()> {
        self.move_cursor_to_bottom(term)?;
        term.inner.clear_line()?;
        term.inner.clear_last_lines(self.pds().height - 1 + self.empty_padding)?;
        Ok(())
    }

    /// Redraw the screen.
    pub fn redraw(&self, term: &MultilineTerm, _: RenderMode) -> io::Result<()> {
        self.clear_draw(term)?;
        self.draw(term)
    }

    /// Draw the line given an index.
    /// This method does not move the cursor.
    pub fn draw_line(&self, term: &MultilineTerm, index: usize) -> io::Result<()> {
        if let Some(f) = &term.gutter {
            term.inner.write_str(&f(index, term))?;
        }
        term.inner.write_str(&term.buffers[index])
    }

    /// Insert a new line on the screen.
    #[inline]
    pub fn new_line(&self, term: &MultilineTerm) -> io::Result<()> {
        term.inner.write_line("")
    }

        /// Move the current cursor to the last line.
    #[inline]
    pub fn move_cursor_to_bottom(&self, term: &MultilineTerm) -> io::Result<()> {
        self.move_cursor_down(term, self.pds().height - self.pds().cursor.line - 1)
    }

    /// Move the cursor to the end of the current line.
    /// This method is not safe to use if the cursor is not at `line:index`,
    #[inline]
    pub fn move_cursor_to_end(&self, term: &MultilineTerm) -> io::Result<()> {
        let pds = self.pds();
        let len = term.current_line_len();
        if pds.cursor.index > len {
            self.move_cursor_left(term, pds.cursor.index - len)
        } else if pds.cursor.index < len {
            self.move_cursor_right(term, len - pds.cursor.index)
        } else {
            Ok(())
        }
    }

    /// Move the cursor to the beginning of the line.
    #[inline]
    pub fn move_cursor_to_start(&self, term: &MultilineTerm) -> io::Result<()> {
        self.move_cursor_left(term, term.cursor.index)?;
        self.update_pds(|pds| pds.cursor.index = 0);
        Ok(())
    }

    /// Move the cursor one line up.
    #[inline]
    pub fn move_cursor_up(&self, term: &MultilineTerm, n: usize) -> io::Result<()> {
        term.inner.move_cursor_up(n)?;
        self.update_pds(|pds| pds.cursor.line -= n);
        Ok(())
    }

    /// Move the cursor one line down.
    #[inline]
    pub fn move_cursor_down(&self, term: &MultilineTerm, n: usize) -> io::Result<()> {
        term.inner.move_cursor_down(n)?;
        self.update_pds(|pds| pds.cursor.line += n);
        Ok(())
    }

    /// Move the cursor leftward using nondestructive backspaces.
    #[inline]
    pub fn move_cursor_left(&self, term: &MultilineTerm, n: usize) -> io::Result<()> {
        term.inner.move_cursor_left(n)?;
        self.update_pds(|pds| pds.cursor.index -= n);
        Ok(())
    }

    /// Move the cursor rightward.
    #[inline]
    pub fn move_cursor_right(&self, term: &MultilineTerm, n: usize) -> io::Result<()> {
        term.inner.move_cursor_right(n)?;
        self.update_pds(|pds| pds.cursor.index += n);
        Ok(())
    }
}