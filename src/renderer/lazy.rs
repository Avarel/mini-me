use std::io::stdout;

use crossterm::{cursor::*, QueueableCommand, Result};

use super::{full::FullRenderer, Renderer, RenderData};

#[derive(Default)]
pub struct LazyRenderer<'a> {
    /// The lazy renderer wraps around a full renderer, using its methods when necessary.
    full: FullRenderer<'a>,
    #[doc(hidden)]
    prange: (usize, usize),
    #[doc(hidden)]
    pbuf: Vec<String>,
}

impl Renderer for LazyRenderer<'_> {
    fn draw(&mut self, data: RenderData) -> Result<()> {
        if data.buffers.is_empty() {
            self.pbuf = vec![String::new()];
        } else {
            self.pbuf = data.buffers.clone();
        }
        self.full.draw(data)?;
        self.flush()
    }

    fn redraw(&mut self, data: RenderData) -> Result<()> {
        match self.find_diff(&data) {
            Diff::NoChange => Ok(()),
            Diff::RedrawCursor => {
                self.full.draw_cursor(&data)?;
                self.flush()
            }
            Diff::RedrawLine(line) => {
                self.redraw_line(&data, line)?;
                self.flush()
            }
            Diff::RedrawAll => {
                stdout().queue(Hide)?;
                self.full.move_cursor_up(self.full.draw_state.cursor.line)?;
                self.draw(data)?;
                stdout().queue(Show)?;
                self.flush()
            }
        }
    }

    fn clear_line(&mut self) -> Result<()> {
        self.full.clear_line()?;
        self.pbuf[self.full.draw_state.cursor.line].clear();
        Ok(())
    }

    fn clear_draw(&mut self) -> Result<()> {
        self.pbuf.clear();
        self.full.clear_draw()
    }

    fn flush(&mut self) -> Result<()> {
        self.full.flush()
    }
}

impl<'w> LazyRenderer<'w> {
    pub fn wrap(renderer: FullRenderer<'w>) -> Self {
        unimplemented!();
        // Self {
        //     full: renderer,
        //     pbuf: Vec::new(),
        // }
    }

    fn find_diff(&mut self, term: &RenderData) -> Diff {
        //if self.full

        let old = &self.pbuf;
        let new = term.buffers;
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
            0 if self.full.draw_state.cursor != *term.cursor => Diff::RedrawCursor,
            0 => Diff::NoChange,
            1 => Diff::RedrawLine(line),
            _ => Diff::RedrawAll,
        }
    }

    fn redraw_line(&mut self, term: &RenderData, line: usize) -> Result<()> {
        if !(self.full.draw_state.buffer_start..self.full.draw_state.buffer_start+self.full.draw_state.height).contains(&line) {
            return Ok(());
        }

        self.full.move_cursor_to_line(line)?;
        self.full.draw_line(term, line)?;

        let buf = term.buffers[line].clone();
        self.full.draw_state.cursor.index = buf.len();
        self.pbuf[line] = buf;

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