use crate::MultilineTerm;

use std::cell::RefCell;
use std::io::stdout;

use crossterm::{cursor::*, QueueableCommand, Result};

use super::{full::FullRenderer, Renderer};

#[derive(Default)]
pub struct LazyRenderer {
    /// The lazy renderer wraps around a full renderer, using its methods when necessary.
    full: FullRenderer,
    #[doc(hidden)]
    pbuf: RefCell<Vec<String>>,
}

impl Renderer for LazyRenderer {
    fn draw(&self, term: &MultilineTerm) -> Result<()> {
        self.full.draw(term)?;
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
                self.full.draw_cursor(term)?;
                self.flush()
            }
            Diff::RedrawLine(line) => {
                self.redraw_line(term, line)?;
                self.flush()
            }
            Diff::RedrawAll => {
                stdout().queue(Hide)?;
                self.full.move_cursor_up(self.full.pds().cursor.line)?;
                self.draw(term)?;
                stdout().queue(Show)?;
                self.flush()
            }
        }
    }

    fn clear_line(&self) -> Result<()> {
        self.full.clear_line()?;
        self.pbuf.borrow_mut()[self.full.pds().cursor.line as usize].clear();
        Ok(())
    }

    fn clear_draw(&self) -> Result<()> {
        self.pbuf.borrow_mut().clear();
        self.full.clear_draw()
    }

    fn flush(&self) -> Result<()> {
        self.full.flush()
    }
}

impl LazyRenderer {
    pub fn wrap(renderer: FullRenderer) -> Self {
        Self {
            full: renderer,
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
            0 if self.full.pds().cursor != *term.cursor() => Diff::RedrawCursor,
            0 => Diff::NoChange,
            1 => Diff::RedrawLine(line),
            _ => Diff::RedrawAll,
        }
    }

    fn redraw_line(&self, term: &MultilineTerm, line: usize) -> Result<()> {
        self.full.move_cursor_to_line(line)?;
        self.full.draw_line(term, line)?;

        let buf = term.buffers()[line as usize].clone();
        self.full.update_pds(|pds| pds.cursor.index = buf.len());
        self.pbuf.borrow_mut()[line as usize] = buf;

        self.full.draw_cursor(term)
    }
}

#[derive(Debug)]
enum Diff {
    NoChange,
    RedrawCursor,
    RedrawLine(usize),
    RedrawAll,
}
