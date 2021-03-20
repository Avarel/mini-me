use std::convert::TryInto;
use std::io::{stdout, Write};

use super::RenderData;
use crate::editor::Cursor;

use crossterm::{
    cursor::*,
    terminal::{Clear, ClearType},
    QueueableCommand, Result,
};

use super::Renderer;

pub struct FullRenderer<'b> {
    write: &'b mut dyn Write,
    draw_state: DrawState,
    /// Formatter.
    formatter: Option<&'b dyn Fn(usize, &RenderData) -> String>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
/// Contains information about the cursor and the height
/// of the last frame drawn.
pub(super) struct DrawState {
    // Initial index of buffer.
    pub buffer_start: usize,
    // How much of the buffer drawn.
    pub height: usize,
    // Position of the cursor RELATIVE TO THE TERMINAL
    // to the start of the drawn frame.
    pub cursor: Cursor,
}

impl Renderer for FullRenderer<'_> {
    /// Draw the prompt.
    fn draw(&mut self, data: RenderData) -> Result<()> {
        // Handle empty buffer.
        if data.line_count() == 0 {
            if let Some(f) = &self.formatter {
                let string = &f(0, &data);
                self.write_str(string)?;
            }
            self.draw_state.height = 1;
            return self.flush();
        }

        let (low, high) = self.calculate_draw_range(&data);

        // Print out the contents.
        for i in low..high {
            self.draw_line(&data, i)?;
            if i < high - 1 {
                // The last line should not have any new-line attached to it.
                self.write_str("\n")?;
            }
        }

        self.write.queue(Clear(ClearType::FromCursorDown))?;

        self.draw_state.buffer_start = low;
        self.draw_state.height = high - low;
        self.draw_state.cursor.line = high - low - 1;
        self.draw_state.cursor.index = data.line(high - 1).len();

        self.draw_cursor(&data)?;
        self.flush()
        // Ok((low, high))
    }

    /// Clear the drawn prompt on the screen.
    fn clear_draw(&mut self) -> Result<()> {
        self.move_cursor_to_bottom()?;
        self.clear_line()?;

        self.move_cursor_up(self.draw_state.height - 1)?;
        self.write.queue(Clear(ClearType::FromCursorDown))?;

        self.draw_state = DrawState::default();

        Ok(())
    }

    /// Clear the line on the current cursor.
    fn clear_line(&mut self) -> Result<()> {
        self.write.queue(Clear(ClearType::CurrentLine))?;
        self.cursor_to_lmargin()?;
        self.draw_state.cursor.index = 0;

        Ok(())
    }

    /// Redraw the screen.
    fn redraw(&mut self, data: RenderData) -> Result<()> {
        self.write.queue(Hide)?;
        self.move_cursor_up(self.draw_state.cursor.line)?;
        self.draw(data)?;
        self.write.queue(Show)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.write.flush()?;
        Ok(())
    }
}

impl<'w> FullRenderer<'w> {
    pub fn render_to(write: &'w mut dyn Write) -> Self {
        FullRenderer {
            write,
            ..Default::default()
        }
    }

    pub fn render_with_formatter<F: Fn(usize, &RenderData) -> String>(f: &'w F) -> Self {
        FullRenderer {
            formatter: Some(f),
            ..Default::default()
        }
    }

    pub fn render_with_formatter_to<F: Fn(usize, &RenderData) -> String>(
        write: &'w mut dyn Write,
        f: &'w F,
    ) -> Self {
        FullRenderer {
            write,
            formatter: Some(f),
            ..Default::default()
        }
    }

    fn write_str(&mut self, s: &str) -> Result<()> {
        write!(self.write, "{}", s)?;
        Ok(())
    }

    fn calculate_draw_range(&self, data: &RenderData) -> (usize, usize) {
        if let Ok((_, rows)) = crossterm::terminal::size() {
            // Rows of the terminal.
            let term_rows: usize = rows.try_into().unwrap();
            // let draw_rows = draw_rows - 1; Useful? Always leave 1 line on the top.
            // Rows of the data to draw.
            let data_rows = data.line_count();
            // Current line of the data.
            let line = data.cursor().line;
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
    pub fn draw_cursor(&mut self, data: &RenderData) -> Result<()> {
        self.move_cursor_to_line(data.cursor().line)?;
        self.move_cursor_to_index(data.cursor().index.min(data.current_line().len()))
    }

    /// Draw the line given an index.
    /// This method does not move the cursor.
    pub fn draw_line(&mut self, data: &RenderData, line: usize) -> Result<()> {
        self.cursor_to_lmargin()?;

        if let Some(f) = self.formatter {
            let string = &f(line, data);
            self.write_str(string)?;
        } else {
            self.write_str(&data.line(line))?;
        }

        // self.write_str(&term.buffers[line])?;
        self.write.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }

    /// Move the current cursor to the last line.
    #[inline]
    pub fn move_cursor_to_bottom(&mut self) -> Result<()> {
        self.move_cursor_down(self.draw_state.height - self.draw_state.cursor.line - 1)
    }

    pub fn move_cursor_to_line(&mut self, line: usize) -> Result<()> {
        let pds_line = self.draw_state.cursor.line + self.draw_state.buffer_start;

        if pds_line > line {
            self.move_cursor_up(pds_line - line)
        } else if pds_line < line {
            self.move_cursor_down(line - pds_line)
        } else {
            Ok(())
        }
    }

    pub fn move_cursor_to_index(&mut self, index: usize) -> Result<()> {
        let pds_index = self.draw_state.cursor.index;

        if index < pds_index {
            self.move_cursor_left(pds_index - index)
        } else if index > pds_index {
            self.move_cursor_right(index - pds_index)
        } else {
            Ok(())
        }
    }

    /// Move the cursor to the beginning of the line.
    #[inline]
    pub fn move_cursor_to_start(&mut self, data: &RenderData) -> Result<()> {
        self.move_cursor_left(data.cursor().index)?;
        Ok(())
    }

    /// Move the curser to the terminal left margin.
    #[doc(hidden)]
    fn cursor_to_lmargin(&mut self) -> Result<()> {
        self.write.queue(MoveToColumn(0))?;
        Ok(())
    }

    /// Move the cursor one line up.
    #[inline]
    pub fn move_cursor_up(&mut self, n: usize) -> Result<()> {
        if n != 0 {
            self.write
                .queue(MoveUp(n.try_into().unwrap_or(std::u16::MAX)))?;
            self.draw_state.cursor.line -= n;
        }
        Ok(())
    }

    /// Move the cursor one line down.
    #[inline]
    pub fn move_cursor_down(&mut self, n: usize) -> Result<()> {
        if n != 0 {
            self.write
                .queue(MoveDown(n.try_into().unwrap_or(std::u16::MAX)))?;
            self.draw_state.cursor.line += n;
        }
        Ok(())
    }

    /// Move the cursor leftward using nondestructive backspaces.
    #[inline]
    pub fn move_cursor_left(&mut self, n: usize) -> Result<()> {
        if n != 0 {
            self.write
                .queue(MoveLeft(n.try_into().unwrap_or(std::u16::MAX)))?;
            self.draw_state.cursor.index -= n;
        }
        Ok(())
    }

    /// Move the cursor rightward.
    #[inline]
    pub fn move_cursor_right(&mut self, n: usize) -> Result<()> {
        if n != 0 {
            self.write
                .queue(MoveRight(n.try_into().unwrap_or(std::u16::MAX)))?;
            self.draw_state.cursor.index += n;
        }
        Ok(())
    }
}

impl Default for FullRenderer<'_> {
    fn default() -> Self {
        // Since stdout() lives for the entirety of the process.
        // This is safe since the handle will always be valid.
        // The only time where it may die is during shutdown.
        let out = Box::new(stdout());
        FullRenderer {
            write: Box::leak(out),
            draw_state: DrawState::default(),
            formatter: None,
        }
    }
}
