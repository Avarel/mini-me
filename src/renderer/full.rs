use std::io::{self, stdout, Write};
use std::{convert::TryInto, io::Stdout};

use super::{margin::{Margin, NoGutter}, RenderData, Renderer};
use crate::Cursor;

use crossterm::{
    cursor::*,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    QueueableCommand, Result,
};

pub struct CrosstermRenderer<'b, W, M> {
    write: &'b mut W,
    margin: M,
    draw_state: DrawState,
}

impl<W, M> Drop for CrosstermRenderer<'_, W, M> {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
/// Contains information about the cursor and the height
/// of the last frame drawn.
struct DrawState {
    height: usize,
    // Anchor of the box
    anchor: Cursor,
    low: usize,
    high: usize,
    // Box cursor relative to the anchor
    cursor: Cursor,
}

impl<W: Write, M: Margin<W>> Renderer for CrosstermRenderer<'_, W, M> {
    /// Draw the prompt.
    fn draw(&mut self, data: RenderData) -> Result<()> {
        let (low, high) = self.calculate_draw_range(&data);

        self.draw_state = DrawState::default();

        self.draw_header(|w| write!(w, "      ╭─── Input Prompt ─────────"))?;

        // Print out the contents.
        for i in low..high {
            self.draw_line(&data, i)?;
            if i < high - 1 {
                // The last line should not have any new-line attached to it.
                self.write.write(b"\n")?;
            }
        }

        self.draw_state.anchor.col = self.margin.width();
        self.draw_state.low = low;
        self.draw_state.high = high;
        self.draw_state.height += high - low;
        self.draw_state.cursor.ln = high - low - 1;
        self.draw_state.cursor.col = data.line(high - 1).len();

        self.write.queue(Clear(ClearType::FromCursorDown))?;

        self.draw_footer(|w| {
            write!(
                w,
                "      ╰──┤ Lines: {} ├─┤ Chars: {} ├─┤ Ln: {}, Col: {}",
                data.line_count(),
                data.char_count(),
                data.cursor().ln,
                data.cursor().col.min(data.current_line().len())
            )
        })?;

        self.draw_cursor(&data)?;
        self.flush()
    }

    /// Clear the drawn prompt on the screen.
    fn clear_draw(&mut self) -> Result<()> {
        self.move_to_frame_base()?;
        self.write.queue(Clear(ClearType::FromCursorDown))?;

        self.draw_state = DrawState::default();

        Ok(())
    }

    /// Redraw the screen.
    fn redraw(&mut self, data: RenderData) -> Result<()> {
        // self.write.queue(Hide)?;
        self.move_to_frame_base()?;
        self.draw(data)?;
        // self.write.queue(Show)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.write.flush()?;
        Ok(())
    }

    fn finish(mut self) -> Result<()> {
        self.clear_draw()?;
        self.flush()
    }
}

const RESERVED_ROWS: usize = 2;

impl<'w, W: Write> CrosstermRenderer<'w, W, NoGutter> {
    pub fn render_to(write: &'w mut W) -> Self {
        enable_raw_mode().unwrap();
        CrosstermRenderer {
            write,
            draw_state: DrawState::default(),
            margin: NoGutter,
        }
    }
}

impl<'w, W: Write, M: Margin<W>> CrosstermRenderer<'w, W, M> {
    fn draw_header(&mut self, mut f: impl FnMut(&mut W) -> io::Result<()>) -> Result<()> {
        self.draw_state.height += 1;
        self.draw_state.anchor.ln += 1;

        self.cursor_to_left_term_edge()?;
        f(self.write)?;
        self.write.queue(Clear(ClearType::UntilNewLine))?;
        self.write.write(b"\n")?;
        Ok(())
    }

    fn draw_footer(&mut self, mut f: impl FnMut(&mut W) -> io::Result<()>) -> Result<()> {
        self.draw_state.height += 1;

        self.write.write(b"\n")?;
        self.cursor_to_left_term_edge()?;
        f(self.write)?;
        self.write.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }

    // region: to factor out as customizations

    // /// Clear the line on the current cursor.
    // fn clear_line(&mut self) -> Result<()> {
    //     self.write.queue(Clear(ClearType::CurrentLine))?;
    //     self.cursor_to_lmargin()?;
    //     self.draw_state.cursor.col = 0;

    //     Ok(())
    // }

    pub fn render_to_with(write: &'w mut W, margin: M) -> Self {
        enable_raw_mode().unwrap();
        CrosstermRenderer {
            write,
            draw_state: DrawState::default(),
            margin,
        }
    }

    fn calculate_draw_range(&self, data: &RenderData) -> (usize, usize) {
        if let Ok((_, rows)) = crossterm::terminal::size() {
            // Rows of the terminal.
            let term_rows: usize = rows.try_into().unwrap();
            let term_rows = term_rows - RESERVED_ROWS;
            // Rows of the data to draw.
            let data_rows = data.line_count();
            // Current line of the data.
            let line = data.cursor().ln;
            if data_rows > term_rows {
                return if line + term_rows / 2 >= data_rows {
                    // Anchor to the bottom.
                    // low = data_rows - term_rows;
                    (data_rows - term_rows, data.line_count())
                } else if term_rows / 2 > line {
                    // Anchor to the top.
                    (0, term_rows)
                } else {
                    // Anchor so that the cursor is in the middle of the draw.
                    (line - term_rows / 2, line + term_rows / 2 + term_rows % 2)
                };
            }
        }
        (0, data.line_count())
    }

    // Move to the base of the frame (not the anchor).
    fn move_to_frame_base(&mut self) -> Result<()> {
        let up_offset = self.draw_state.anchor.ln + self.draw_state.cursor.ln;
        self.write.queue(MoveUp(Self::usize_to_u16(up_offset)))?;
        self.write.queue(MoveToColumn(0))?;
        Ok(())
    }

    // Position the cursor right after drawing a frame
    // (assuming no other cursor adjustments made).
    fn draw_cursor(&mut self, data: &RenderData) -> Result<()> {
        // Move to the correct row.
        let line = data.cursor().ln;
        let frame_height = self.draw_state.height;
        let relative_ln = line - self.draw_state.low;
        let up_offset = frame_height - 1 - self.draw_state.anchor.ln - relative_ln;
        // self.move_cursor_up(up_offset)?;
        self.write.queue(MoveUp(Self::usize_to_u16(up_offset)))?;

        // Move to the correct column.
        let col = data.cursor().col.min(data.current_line().len());
        let n = self.draw_state.anchor.col + col + 1;
        self.write.queue(MoveToColumn(Self::usize_to_u16(n)))?;

        self.draw_state.cursor.ln = relative_ln;
        self.draw_state.cursor.col = n;

        Ok(())
    }

    /// Draw the line given an index.
    /// This method does not move the cursor.
    fn draw_line(&mut self, data: &RenderData, line: usize) -> Result<()> {
        self.cursor_to_left_term_edge()?;

        self.margin.draw_margin(self.write, line, data)?;

        data.write_line(line, self.write)?;

        self.write.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }

    /// Move the curser to the terminal left margin.
    #[doc(hidden)]
    fn cursor_to_left_term_edge(&mut self) -> Result<()> {
        self.write.queue(MoveToColumn(0))?;
        Ok(())
    }

    fn usize_to_u16(n: usize) -> u16 {
        n.try_into().unwrap_or(std::u16::MAX)
    }
}

impl Default for CrosstermRenderer<'_, Stdout, NoGutter> {
    fn default() -> Self {
        // Since stdout() lives for the entirety of the process.
        // This is safe since the handle will always be valid.
        // The only time where it may die is during shutdown.
        let out = Box::new(stdout());
        CrosstermRenderer::render_to_with(Box::leak(out), NoGutter)
    }
}
