use std::io::{self, stdout, Write};
use std::{convert::TryInto, io::Stdout};

use super::{RenderData, Renderer};
use crate::editor::Cursor;

use crossterm::{
    cursor::*,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    QueueableCommand, Result,
};

pub struct CrosstermRenderer<'b, W> {
    write: &'b mut W,
    draw_state: DrawState,
}

impl<W> Drop for CrosstermRenderer<'_, W> {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
/// Contains information about the cursor and the height
/// of the last frame drawn.
struct DrawState {
    // Initial index of buffer.
    line_start: usize,
    // How much of the buffer drawn.
    height: usize,
    // Position of the cursor relative to the end of the margin
    // to the start of the drawn frame.
    cursor: Cursor,
}

impl<W: Write> Renderer for CrosstermRenderer<'_, W> {
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

        self.draw_state.line_start += low;
        self.draw_state.height += high - low;
        self.draw_state.cursor.ln += high - low - 1;
        self.draw_state.cursor.col += data.line(high - 1).len();

        self.write.queue(Clear(ClearType::FromCursorDown))?;

        self.draw_footer(|w| {
            write!(
                w,
                "      ╰─── Lines: {:<5} Chars: {:<5} Ln: {}, Col: {}",
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
        self.move_cursor_up(self.draw_state.cursor.ln)?;
        self.cursor_to_lmargin()?;
        self.write.queue(Clear(ClearType::FromCursorDown))?;

        self.draw_state = DrawState::default();

        Ok(())
    }

    /// Redraw the screen.
    fn redraw(&mut self, data: RenderData) -> Result<()> {
        // self.write.queue(Hide)?;
        self.move_cursor_up(self.draw_state.cursor.ln)?;
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

// region: to factor out as customizations
const RESERVED_ROWS: usize = 2;
const GUTTER: usize = 5;
const MARGIN: usize = 3;
const MARGIN_STR: &[&str] = &[" │ ", " ┃ "];
const TOTAL_MARGIN: usize = GUTTER + MARGIN;

impl<'w, W: Write> CrosstermRenderer<'w, W> {
    fn draw_header(&mut self, mut f: impl FnMut(&mut W) -> io::Result<()>) -> Result<()> {
        self.draw_state.height += 1;
        self.draw_state.line_start -= 1;
        self.draw_state.cursor.ln += 1;

        self.cursor_to_lmargin()?;
        f(self.write)?;
        self.write.queue(Clear(ClearType::UntilNewLine))?;
        self.write.write(b"\n")?;
        Ok(())
    }

    fn draw_footer(&mut self, mut f: impl FnMut(&mut W) -> io::Result<()>) -> Result<()> {
        self.draw_state.height += 1;
        self.draw_state.cursor.ln += 1;
        self.draw_state.cursor.col = 0;

        self.write.write(b"\n")?;
        self.cursor_to_lmargin()?;
        f(self.write)?;
        self.write.queue(Clear(ClearType::UntilNewLine))?;
        self.write
            .queue(MoveToColumn((TOTAL_MARGIN + 1).try_into().unwrap()))?;
        Ok(())
    }

    fn draw_margin(&mut self, line_idx: usize, data: &RenderData) -> Result<()> {
        write!(self.write, "{:>width$}", line_idx + 1, width = GUTTER)?;
        let z = if line_idx == data.cursor().ln { 1 } else { 0 };
        self.write.write(MARGIN_STR[z].as_bytes())?;
        Ok(())
    }
    // region: to factor out as customizations

    /// Clear the line on the current cursor.
    fn clear_line(&mut self) -> Result<()> {
        self.write.queue(Clear(ClearType::CurrentLine))?;
        self.cursor_to_lmargin()?;
        self.draw_state.cursor.col = 0;

        Ok(())
    }

    pub fn render_to(write: &'w mut W) -> Self {
        enable_raw_mode().unwrap();
        CrosstermRenderer {
            write,
            draw_state: DrawState::default(),
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

    // Position the cursor.
    fn draw_cursor(&mut self, data: &RenderData) -> Result<()> {
        self.move_cursor_to_line(data.cursor().ln)?;
        self.move_cursor_to_index(data.cursor().col.min(data.current_line().len()))
    }

    /// Draw the line given an index.
    /// This method does not move the cursor.
    fn draw_line(&mut self, data: &RenderData, line: usize) -> Result<()> {
        self.cursor_to_lmargin()?;

        self.draw_margin(line, data)?;

        data.write_line(line, self.write)?;

        self.write.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }

    /// Move the current cursor to the last line.
    fn move_cursor_to_bottom(&mut self) -> Result<()> {
        self.move_cursor_down(self.draw_state.height - self.draw_state.cursor.ln - 1)
    }

    fn move_cursor_to_line(&mut self, line: usize) -> Result<()> {
        let pds_line = self.draw_state.cursor.ln + self.draw_state.line_start;

        if pds_line > line {
            self.move_cursor_up(pds_line - line)
        } else if pds_line < line {
            self.move_cursor_down(line - pds_line)
        } else {
            Ok(())
        }
    }

    fn move_cursor_to_index(&mut self, index: usize) -> Result<()> {
        let pds_index = self.draw_state.cursor.col;

        if index < pds_index {
            self.move_cursor_left(pds_index - index)
        } else if index > pds_index {
            self.move_cursor_right(index - pds_index)
        } else {
            Ok(())
        }
    }

    /// Move the cursor to the beginning of the line.
    fn move_cursor_to_start(&mut self, data: &RenderData) -> Result<()> {
        self.move_cursor_left(data.cursor().col)?;
        Ok(())
    }

    /// Move the curser to the terminal left margin.
    #[doc(hidden)]
    fn cursor_to_lmargin(&mut self) -> Result<()> {
        self.write.queue(MoveToColumn(0))?;
        Ok(())
    }

    fn usize_to_u16(n: usize) -> u16 {
        n.try_into().unwrap_or(std::u16::MAX)
    }

    /// Move the cursor one line up.
    fn move_cursor_up(&mut self, n: usize) -> Result<()> {
        if n != 0 {
            self.write.queue(MoveUp(Self::usize_to_u16(n)))?;
            self.draw_state.cursor.ln -= n;
        }
        Ok(())
    }

    /// Move the cursor one line down.
    fn move_cursor_down(&mut self, n: usize) -> Result<()> {
        if n != 0 {
            self.write.queue(MoveDown(Self::usize_to_u16(n)))?;
            self.draw_state.cursor.ln += n;
        }
        Ok(())
    }

    /// Move the cursor leftward using nondestructive backspaces.
    fn move_cursor_left(&mut self, n: usize) -> Result<()> {
        if n != 0 {
            self.write.queue(MoveLeft(Self::usize_to_u16(n)))?;
            self.draw_state.cursor.col -= n;
        }
        Ok(())
    }

    /// Move the cursor rightward.
    fn move_cursor_right(&mut self, n: usize) -> Result<()> {
        if n != 0 {
            self.write.queue(MoveRight(Self::usize_to_u16(n)))?;
            self.draw_state.cursor.col += n;
        }
        Ok(())
    }
}

impl Default for CrosstermRenderer<'_, Stdout> {
    fn default() -> Self {
        // Since stdout() lives for the entirety of the process.
        // This is safe since the handle will always be valid.
        // The only time where it may die is during shutdown.
        let out = Box::new(stdout());
        CrosstermRenderer::render_to(Box::leak(out))
    }
}
