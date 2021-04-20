use std::{
    convert::TryInto,
    io::{stdout, Stdout, Write},
};

use super::{Editor, Renderer, styles::{NoStyle, Style}};
use crate::{editor::selection::Cursor, Result};

use crossterm::{
    cursor::*,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};
use raw_mode::RawModeGuard;

mod raw_mode {
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

    use super::Result;

    pub struct RawModeGuard(());

    impl RawModeGuard {
        pub fn acquire() -> Result<RawModeGuard> {
            enable_raw_mode()?;
            Ok(Self(()))
        }
    }

    impl Drop for RawModeGuard {
        fn drop(&mut self) {
            disable_raw_mode().unwrap();
        }
    }
}

pub struct CrosstermRenderer<'w, W, S> {
    _guard: RawModeGuard,
    write: &'w mut W,
    style: S,
    draw_state: DrawState,
    max_height: Option<usize>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
/// Contains information about the cursor and the height
/// of the last frame drawn.
struct DrawState {
    altscreen: bool,
    height: usize,
    // Anchor of the box
    anchor: Cursor,
    low: usize,
    high: usize,
    // Box cursor relative to the anchor
    cursor: Cursor,
}

impl<W, S> Renderer for CrosstermRenderer<'_, W, S>
where
    W: Write,
    S: Style<W>
{
    /// Draw the prompt.
    fn draw(&mut self, data: &Editor) -> Result<()> {
        if self.draw_state.altscreen {
            self.write.queue(MoveTo(0, 0))?;
        } else {
            self.move_to_frame_base()?;
        }

        let (low, high, term_rows) = self.calculate_draw_range(&data);

        if term_rows == 0 {
            return Ok(());
        }

        if data.altscreen && !self.draw_state.altscreen {
            self.write.queue(EnterAlternateScreen)?;
        } else if !data.altscreen && self.draw_state.altscreen {
            self.write.queue(LeaveAlternateScreen)?;
        }

        self.draw_state = DrawState::default();
        self.draw_state.altscreen = data.altscreen;

        self.draw_header(&data)?;
        self.draw_range(&data, low, high, term_rows)?;
        self.draw_footer(&data)?;
        self.write.queue(Clear(ClearType::FromCursorDown))?;

        self.draw_cursor(&data)?;
        self.flush()
    }

    /// Clear the drawn prompt on the screen.
    fn clear_draw(&mut self) -> Result<()> {
        if self.draw_state.altscreen {
            self.write.queue(MoveTo(0, 0))?;
            self.write.queue(Clear(ClearType::All))?;
        } else {
            self.move_to_frame_base()?;
            self.write.queue(Clear(ClearType::FromCursorDown))?;
        }

        self.draw_state = DrawState::default();

        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.write.flush()?;
        Ok(())
    }

    fn finish(mut self) -> Result<()> {
        self.clear_draw()?;

        // if self.draw_state.altscreen {
        self.write.queue(LeaveAlternateScreen)?;
        // }

        self.flush()
    }
}

impl<'w, W> CrosstermRenderer<'w, W, NoStyle> {
    pub fn render_to(write: &'w mut W) -> Self {
        CrosstermRenderer {
            _guard: RawModeGuard::acquire().unwrap(),
            write,
            draw_state: DrawState::default(),
            style: NoStyle,
            max_height: None
        }
    }

    pub fn max_height(self, max_height: Option<usize>) -> Self {
        Self {
            max_height,
            ..self
        }
    }
}

impl<'w, W, S> CrosstermRenderer<'w, W, S> {
    pub fn with_style<S2>(self, style: S2) -> CrosstermRenderer<'w, W, S2> {
        CrosstermRenderer {
            _guard: self._guard,
            write: self.write,
            draw_state: self.draw_state,
            style,
            max_height: self.max_height
        }
    }
}

impl<'w, W, S> CrosstermRenderer<'w, W, S>
where
    W: Write,
    S: Style<W>
{
    fn calculate_draw_range(&self, data: &Editor) -> (usize, usize, usize) {
        if let Ok((_, rows)) = crossterm::terminal::size() {
            // Rows of the terminal.
            let max_height = if !data.altscreen {
                self.max_height
            } else {
                None
            };
            let term_rows = max_height
                .unwrap_or(usize::MAX)
                .min(rows.try_into().unwrap())
                .saturating_sub(self.style.header_rows())
                .saturating_sub(self.style.footer_rows());
            if term_rows == 0 {
                return (0, 0, 0);
            }
            // Rows of the data to draw.
            let data_rows = data.line_count();
            // Current line of the data.
            let line = data.selection.focus.ln;
            if data_rows > term_rows {
                let (low, high) = if line >= self.draw_state.high {
                    (line - term_rows + 1, line + 1)
                } else if line < self.draw_state.low {
                    (line, line + term_rows)
                } else {
                    (self.draw_state.low, self.draw_state.high)
                };
                (low, high.min(data_rows), term_rows)
            } else {
                (0, data.line_count(), term_rows)
            }
        } else {
            (0, data.line_count(), 0)
        }
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
    fn draw_cursor(&mut self, data: &Editor) -> Result<()> {
        // Move to the correct row.
        let line = data.selection.focus.ln;
        let frame_height = self.draw_state.height;
        let relative_ln = line - self.draw_state.low;
        let up_offset = frame_height - 1 - self.draw_state.anchor.ln - relative_ln;
        // self.move_cursor_up(up_offset)?;
        self.write.queue(MoveUp(Self::usize_to_u16(up_offset)))?;

        // Move to the correct column.
        let col = data.selection.focus.col.min(data.curr_ln_len());
        let n = self.draw_state.anchor.col + col + 1;
        self.write.queue(MoveToColumn(Self::usize_to_u16(n)))?;

        self.draw_state.cursor.ln = relative_ln;
        self.draw_state.cursor.col = n;

        Ok(())
    }

    fn draw_header(&mut self, data: &Editor) -> Result<()> {
        self.draw_state.height += self.style.header_rows();
        self.draw_state.anchor.ln += self.style.header_rows();

        self.cursor_to_left_term_edge()?;
        self.style.draw_header(self.write, data)?;
        if self.style.header_rows() > 0 {
            self.write.write(b"\n")?;
        }
        Ok(())
    }

    /// Draw the line given an index.
    /// This method does not move the cursor.
    fn draw_line(&mut self, data: &Editor, line: usize) -> Result<()> {
        self.cursor_to_left_term_edge()?;

        self.style.draw_gutter(self.write, line, data)?;
        if line < data.line_count() {
            data.write_line(line, self.write)?;
        }
        self.write.queue(Clear(ClearType::UntilNewLine))?;

        Ok(())
    }

    fn draw_footer(&mut self, data: &Editor) -> Result<()> {
        self.draw_state.height += self.style.footer_rows();

        self.cursor_to_left_term_edge()?;
        if self.style.footer_rows() > 0 {
            self.write.write(b"\n")?;
        }
        // write!(self.write, "{} {} {}", self.draw_state.low, self.draw_state.high, data.cursor.ln)?;
        self.style.draw_footer(self.write, data)?;
        Ok(())
    }

    fn draw_range(
        &mut self,
        data: &Editor,
        low: usize,
        high: usize,
        term_rows: usize,
    ) -> Result<()> {
        // Print out the contents.
        for i in low..high {
            self.draw_line(&data, i)?;
            if i < high - 1 {
                // The last line should not have any new-line attached to it.
                self.write.write(b"\n")?;
            }
        }

        self.draw_state.anchor.col = self.style.gutter_width();
        self.draw_state.low = low;
        self.draw_state.high = high;
        self.draw_state.height += high - low;
        self.draw_state.cursor.ln = high - low - 1;
        self.draw_state.cursor.col = data.line(high - 1).len();

        if data.altscreen {
            for i in high..low + term_rows {
                self.write.write(b"\n")?;
                self.draw_line(&data, i)?;
            }
            self.draw_state.height += low + term_rows - high;
            self.draw_state.cursor.ln += low + term_rows - high;
        }

        Ok(())
    }

    /// Move the curser to the terminal left margin.
    fn cursor_to_left_term_edge(&mut self) -> Result<()> {
        self.write.queue(MoveToColumn(0))?;
        Ok(())
    }

    #[doc(hidden)]
    fn usize_to_u16(n: usize) -> u16 {
        n.try_into().unwrap_or(std::u16::MAX)
    }
}

impl Default for CrosstermRenderer<'static, Stdout, NoStyle> {
    fn default() -> Self {
        let out = Box::new(stdout());
        CrosstermRenderer::render_to(Box::leak(out))
    }
}
