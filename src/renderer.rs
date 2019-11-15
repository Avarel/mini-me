use crate::{Cursor, MultilineTerm};
use std::cell::{Cell, RefCell};
use std::convert::TryInto;
use std::io::{self, stdout, Write};

use crossterm::{
    cursor::*,
    terminal::{Clear, ClearType},
    Output, QueueableCommand, Result,
};

pub trait Renderer {
    fn draw(&self, term: &MultilineTerm) -> Result<()>;
    fn redraw(&self, term: &MultilineTerm) -> Result<()>;
    fn clear_draw(&self) -> Result<()>;
    fn clear_line(&self) -> Result<()>;
    fn flush(&self) -> Result<()>;
}

#[derive(Default)]
pub struct FullRenderer {
    #[doc(hidden)]
    pds: Cell<PreviousDrawState>,
    /// Function to draw the prompt.
    gutter: Option<Box<dyn Fn(usize, &MultilineTerm) -> String>>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreviousDrawState {
    pub height: u16,
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
            return Ok(());
        }

        // Print out the contents.
        for i in 0..term.buffers.len() {
            self.draw_line(term, i)?;
            if i < term.buffers.len() - 1 {
                // The last line should not have any new-line attached to it.
                self.new_line()?;
            }
        }

        self.update_pds(|pds| {
            pds.height = term.buffers.len().try_into().unwrap();
            pds.cursor.line = (term.buffers.len() - 1).try_into().unwrap();
            pds.cursor.index = (term.buffers.last().unwrap().len()).try_into().unwrap();
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
        self.clear_draw()?;
        self.draw(term)
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
    fn update_pds<F: FnOnce(&mut PreviousDrawState)>(&self, f: F) {
        let mut pds = self.pds();
        f(&mut pds);
        self.pds.set(pds);
    }

    #[doc(hidden)]
    fn pds(&self) -> PreviousDrawState {
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
        self.write_str(&term.buffers[line])
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

    pub fn move_cursor_to_line(&self, line: u16) -> Result<()> {
        let pds_line = self.pds().cursor.line;

        if pds_line > line {
            self.move_cursor_up(pds_line - line)
        } else if pds_line < line {
            self.move_cursor_down(line - pds_line)
        } else {
            Ok(())
        }
    }

    pub fn move_cursor_to_index(&self, index: u16) -> Result<()> {
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
    pub fn move_cursor_up(&self, n: u16) -> Result<()> {
        if n == 0 {
            return Ok(());
        }
        stdout().queue(MoveUp(n))?;
        self.update_pds(|pds| pds.cursor.line -= n);
        Ok(())
    }

    /// Move the cursor one line down.
    #[inline]
    pub fn move_cursor_down(&self, n: u16) -> Result<()> {
        if n == 0 {
            return Ok(());
        }
        stdout().queue(MoveDown(n))?;
        self.update_pds(|pds| pds.cursor.line += n);
        Ok(())
    }

    /// Move the cursor leftward using nondestructive backspaces.
    #[inline]
    pub fn move_cursor_left(&self, n: u16) -> Result<()> {
        if n == 0 {
            return Ok(());
        }
        stdout().queue(MoveLeft(n))?;
        self.update_pds(|pds| pds.cursor.index -= n);
        Ok(())
    }

    /// Move the cursor rightward.
    #[inline]
    pub fn move_cursor_right(&self, n: u16) -> Result<()> {
        if n == 0 {
            return Ok(());
        }
        stdout().queue(MoveRight(n))?;
        self.update_pds(|pds| pds.cursor.index += n);
        Ok(())
    }
}

#[derive(Default)]
pub struct LazyRenderer {
    /// The lazy renderer wraps around a full renderer, using its methods when necessary.
    inner: FullRenderer,
    #[doc(hidden)]
    pbuf: RefCell<Vec<String>>,
}

impl Renderer for LazyRenderer {
    fn draw(&self, term: &MultilineTerm) -> Result<()> {
        self.inner.draw(term)?;
        if term.buffers().is_empty() {
            self.pbuf.replace(vec![String::new()]);
        } else {
            self.pbuf.replace(term.buffers().clone());
        }
        self.flush()
    }

    fn redraw(&self, term: &MultilineTerm) -> Result<()> {
        match self.find_diff(term) {
            Diff::NoChange => Ok(()),
            Diff::RedrawCursor => {
                self.inner.draw_cursor(term)?;
                self.flush()
            }
            Diff::RedrawLine(line) => {
                self.redraw_line(term, line.try_into().unwrap())?;
                self.flush()
            }
            Diff::RedrawAll => {
                self.clear_draw()?;
                self.draw(term)
            }
        }
    }

    fn clear_line(&self) -> Result<()> {
        self.inner.clear_line()?;
        self.pbuf.borrow_mut()[self.inner.pds().cursor.line as usize].clear();
        Ok(())
    }

    fn clear_draw(&self) -> Result<()> {
        self.pbuf.borrow_mut().clear();
        self.inner.clear_draw()
    }

    fn flush(&self) -> Result<()> {
        self.inner.flush()
    }
}

impl LazyRenderer {
    pub fn wrap(renderer: FullRenderer) -> Self {
        Self {
            inner: renderer,
            pbuf: RefCell::new(Vec::new()),
        }
    }

    fn find_diff(&self, term: &MultilineTerm) -> Diff {
        let old = self.pbuf.borrow();
        let new = term.buffers();
        if old.len() != new.len() {
            return Diff::RedrawAll;
        }
        let mut changes = 0;
        let mut line = 0;

        for i in 0..old.len() {
            if old[i] != new[i] {
                changes += 1;
                line = i;
            }
        }

        match changes {
            0 if self.inner.pds().cursor != *term.cursor() => Diff::RedrawCursor,
            0 => Diff::NoChange,
            1 => Diff::RedrawLine(line),
            _ => Diff::RedrawAll,
        }
    }

    fn redraw_line(&self, term: &MultilineTerm, line: u16) -> Result<()> {
        self.inner.move_cursor_to_line(line)?;
        self.inner.clear_line()?;
        self.inner.draw_line(term, line.try_into().unwrap())?;

        let buf = term.buffers()[line as usize].clone();
        self.inner
            .update_pds(|pds| pds.cursor.index = buf.len().try_into().unwrap());
        self.pbuf.borrow_mut()[line as usize] = buf;

        self.inner.draw_cursor(term)
    }
}

#[derive(Debug)]
enum Diff {
    NoChange,
    RedrawCursor,
    RedrawLine(usize),
    RedrawAll,
}
