use std::{
    convert::TryInto,
    io::{stdout, Stdout, Write},
};

use super::{
    footer::{Footer, NoFooter},
    header::{Header, NoHeader},
    margin::{Margin, NoMargin},
    RenderData, Renderer,
};
use crate::util::{Cursor, RawModeGuard};

use crossterm::{
    cursor::*,
    terminal::{Clear, ClearType},
    QueueableCommand, Result,
};

pub struct CrosstermRenderer<'b, W, M, H, F> {
    _guard: RawModeGuard,
    write: &'b mut W,
    margin: M,
    header: H,
    footer: F,
    draw_state: DrawState,
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

impl<W: Write, M: Margin<W>, H: Header<W>, F: Footer<W>> Renderer
    for CrosstermRenderer<'_, W, M, H, F>
{
    /// Draw the prompt.
    fn draw(&mut self, data: RenderData) -> Result<()> {
        let (low, high) = self.calculate_draw_range(&data);

        self.draw_state = DrawState::default();

        // self.draw_header(|w| write!(w, "      ╭─── Input Prompt ─────────"))?;
        self.draw_header(&data)?;

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

        self.draw_footer(&data)?;

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

impl<'w, W: Write> DefaultRenderer<'w, W> {
    pub fn render_to(write: &'w mut W) -> Self {
        CrosstermRenderer {
            _guard: RawModeGuard::acquire(),
            write,
            draw_state: DrawState::default(),
            margin: NoMargin,
            header: NoHeader,
            footer: NoFooter,
        }
    }
}

impl<'w, W: Write, M: Margin<W>, H: Header<W>, F: Footer<W>> CrosstermRenderer<'w, W, M, H, F> {
    fn draw_header(&mut self, data: &RenderData) -> Result<()> {
        self.draw_state.height += self.header.height();
        self.draw_state.anchor.ln += self.header.height();

        self.cursor_to_left_term_edge()?;
        self.header.draw(self.write, data)?;
        Ok(())
    }

    fn draw_footer(&mut self, data: &RenderData) -> Result<()> {
        self.draw_state.height += self.footer.height();

        self.cursor_to_left_term_edge()?;
        self.footer.draw(self.write, data)?;
        Ok(())
    }

    fn calculate_draw_range(&self, data: &RenderData) -> (usize, usize) {
        if let Ok((_, rows)) = crossterm::terminal::size() {
            // Rows of the terminal.
            let term_rows: usize = rows.try_into().unwrap();
            let term_rows = term_rows - self.header.height() - self.footer.height();
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

        self.margin.draw(self.write, line, data)?;

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

// region: Swap constructors
impl<'w, W: Write, M1, H, F> CrosstermRenderer<'w, W, M1, H, F> {
    /// Swap out a margin formatter.
    pub fn margin<M2>(self, margin: M2) -> CrosstermRenderer<'w, W, M2, H, F> {
        CrosstermRenderer {
            _guard: self._guard,
            write: self.write,
            draw_state: self.draw_state,
            margin,
            header: self.header,
            footer: self.footer,
        }
    }
}

impl<'w, W: Write, M, H1, F> CrosstermRenderer<'w, W, M, H1, F> {
    /// Swap out a header formatter.
    pub fn header<H2>(self, header: H2) -> CrosstermRenderer<'w, W, M, H2, F> {
        CrosstermRenderer {
            _guard: self._guard,
            write: self.write,
            draw_state: self.draw_state,
            margin: self.margin,
            header,
            footer: self.footer,
        }
    }
}

impl<'w, W: Write, M, H, F1> CrosstermRenderer<'w, W, M, H, F1> {
    /// Swap out a footer formatter.
    pub fn footer<F2>(self, footer: F2) -> CrosstermRenderer<'w, W, M, H, F2> {
        CrosstermRenderer {
            _guard: self._guard,
            write: self.write,
            draw_state: self.draw_state,
            margin: self.margin,
            header: self.header,
            footer,
        }
    }
}

pub type DefaultRenderer<'w, W> = CrosstermRenderer<'w, W, NoMargin, NoHeader, NoFooter>;

impl Default for DefaultRenderer<'static, Stdout> {
    fn default() -> Self {
        // Since stdout() lives for the entirety of the process.
        // This is safe since the handle will always be valid.
        // The only time where it may die is during shutdown.
        let out = Box::new(stdout());
        CrosstermRenderer::render_to(Box::leak(out))
    }
}