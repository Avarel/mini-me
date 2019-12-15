pub mod renderer;

use renderer::{lazy::LazyRenderer, RenderData, Renderer};

pub use crossterm;

use crossterm::{
    event::{read, Event, KeyEvent, KeyCode},
    terminal::{enable_raw_mode, disable_raw_mode},
    Result,
};

/// Multiline abstraction around a terminal.
pub struct MultilineTerm<'w> {
    buffers: Vec<String>,
    cursor: Cursor,
    renderer: Box<dyn 'w + Renderer>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    /// Current line of the cursor.
    pub line: usize,
    /// Current index of the cursor.
    pub index: usize,
}

impl<'w> MultilineTerm<'w> {
    /// Create a builder for `MultilineTerm`.
    #[inline]
    pub fn builder<'b>() -> MultilineTermBuilder<'b> {
        MultilineTermBuilder::default()
    }

    /// Return a mutable reference to the buffer of the terminal.
    #[inline]
    pub fn buffers_mut(&mut self) -> &mut Vec<String> {
        &mut self.buffers
    }

    #[doc(hidden)]
    #[inline]
    fn current_line_len(&self) -> usize {
        self.current_line().len()
    }

    /// Get a reference to current line of the cursor on the buffer.
    /// Unlike `current_line_mut`, this function will not allocate a new string
    /// if the buffer is empty, instead returning an empty string.
    pub fn current_line(&self) -> &str {
        if self.buffers.len() == 0 {
            return "";
        }
        &self.buffers[self.cursor.line]
    }

    /// Get a mutable reference to the current line of the cursor on the buffer.
    ///
    /// ### Warning
    /// This function will allocate a new `String` to the buffer if it is empty.
    pub fn current_line_mut(&mut self) -> &mut String {
        if self.buffers.is_empty() {
            let s = String::new();
            self.buffers.push(s);
            return &mut self.buffers[0];
        }
        let line = self.cursor.line;
        &mut self.buffers[line]
    }

    fn render_data<'b>(buffers: &'b Vec<String>, cursor: &'b Cursor) -> RenderData<'b> {
        RenderData { buffers, cursor }
    }

    /// Read multiple lines of input.
    ///
    /// ### Features
    /// * `Enter` on an empty last line will submit the input.
    /// * `Enter` on a non-empty line will create a new line.
    /// * `Backspace` at the beginning of the line to tappend the content
    ///   of the current line to the previous line.
    ///
    /// The returned result does not include the final empty line or trailing newline.
    pub fn read_multiline(mut self) -> Result<String> {
        self.renderer
            .draw(&Self::render_data(&self.buffers, &self.cursor))?;
        self.renderer.flush()?;

        enable_raw_mode()?;
        
        loop {
            let key_event = read()?;
            match key_event {
                Event::Key(k) => {
                    if self.process_key_event(k)? {
                        continue;
                    } else {
                        break;
                    }
                }
                _ => {}
            }
        }

        // Clear the last empty useless line.
        self.renderer.clear_line()?;
        self.renderer.flush()?;

        // If empty buffer, then return empty string.
        if self.buffers.is_empty() {
            return Ok(String::new());
        }

        // Join the buffers together, putting a `\n` in between each line.
        // Ensure the capacity to avoid reallocations.
        let mut buf =
            String::with_capacity(self.buffers.iter().map(|x| x.len() + 1).sum::<usize>() - 1);
        buf.push_str(&self.buffers[0]);
        for s in &self.buffers[1..] {
            buf.push('\n');
            buf.push_str(&s);
        }

        disable_raw_mode()?;

        Ok(buf)
    }

    /// Return Ok(continue?)
    fn process_key_event(&mut self, event: KeyEvent) -> Result<bool> {
        let code = event.code;
        match code {
            KeyCode::Down => {
                if self.buffers.is_empty() {
                    return Ok(true);
                }
                if self.cursor.line + 1 < self.buffers.len() {
                    self.cursor.line += 1;
                    self.renderer
                        .redraw(&Self::render_data(&self.buffers, &self.cursor))?;
                }
            }
            KeyCode::Up => {
                if self.buffers.is_empty() {
                    return Ok(true);
                }
                if self.cursor.line > 0 {
                    self.cursor.line -= 1;
                    self.renderer
                        .redraw(&Self::render_data(&self.buffers, &self.cursor))?;
                }
            }
            KeyCode::Left => {
                if self.buffers.is_empty() {
                    return Ok(true);
                }
                self.cursor.index = self.ensure_cursor_index();
                if self.cursor.index > 0 {
                    self.cursor.index -= 1;
                } else if self.cursor.line > 0 {
                    // Move to the end of the previous line.
                    self.cursor.line -= 1;
                    self.cursor.index = self.buffers[self.cursor.line].len();
                }
                self.renderer
                    .redraw(&Self::render_data(&self.buffers, &self.cursor))?;
            }
            KeyCode::Right => {
                if self.buffers.is_empty() {
                    return Ok(true);
                }
                self.cursor.index = self.ensure_cursor_index();
                let len = self.current_line().len();
                if self.cursor.index < len {
                    self.cursor.index += 1;
                } else if self.cursor.line + 1 < self.buffers.len() {
                    // Move to the beginning of the next line.
                    self.cursor.line += 1;
                    self.cursor.index = 0;
                }
                self.renderer
                    .redraw(&Self::render_data(&self.buffers, &self.cursor))?;
            }
            KeyCode::Backspace => {
                if self.buffers.is_empty() {
                    return Ok(true);
                }
                self.cursor.index = self.ensure_cursor_index();

                if self.cursor.index > 0 {
                    self.cursor.index = self.delete_char_before_cursor();
                    self.renderer
                        .redraw(&Self::render_data(&self.buffers, &self.cursor))?;
                } else if self.cursor.line > 0 {
                    // Backspace at the beginning of the line, so push the contents of
                    // the current line to the line above it, and remove the line.

                    // Push the content of the current line to the previous line.
                    let cbuf = self.buffers.remove(self.cursor.line);
                    // Change line number.
                    self.cursor.line -= 1;

                    // The cursor should now be at the end of the previous line
                    // before appending the contents of the current line.
                    self.cursor.index = self.current_line_len();
                    self.current_line_mut().push_str(&cbuf);
                    self.renderer
                        .redraw(&Self::render_data(&self.buffers, &self.cursor))?;
                }
            }
            // TODO: Delete key.
            KeyCode::Tab => {
                self.cursor.index = self.ensure_cursor_index();
                let soft = 4 - self.current_line_len() % 4;
                for _ in 0..soft {
                    self.cursor.index = self.insert_char_before_cursor(' ');
                }
                self.renderer
                    .redraw(&Self::render_data(&self.buffers, &self.cursor))?;
            }
            KeyCode::Char(c) => {
                self.cursor.index = self.ensure_cursor_index();
                self.cursor.index = self.insert_char_before_cursor(c);
                self.renderer
                    .redraw(&Self::render_data(&self.buffers, &self.cursor))?;
            }
            // TODO: Add it back in.
            // KeyEvent::Esc => {
            //     // // Quick escape and finish the input.
            //     if self.buffers.len() != 0 {
            //         self.renderer.move_cursor_to_bottom(&self)?;
            //         if self.current_line_len() == 0 {
            //             self.buffers.remove(self.cursor.line);
            //         } else {
            //             self.renderer.new_line(&self)?;
            //         }
            //     }
            //     return Ok(false)
            // }
            KeyCode::Enter => {
                if self.buffers.len() == 0 {
                    return Ok(false);
                } else if self.cursor.line + 1 == self.buffers.len() && self.current_line_len() == 0
                {
                    // Enter on the last line of the prompt which is also empty
                    // finishes the input.

                    // Remove last useless line.
                    self.buffers.remove(self.cursor.line);
                    return Ok(false);
                } else {
                    self.cursor.index = self.ensure_cursor_index();
                    // Split the input after the cursor.
                    let cursor_idx = self.cursor.index;
                    let cbuf = self.current_line_mut();
                    let nbuf = cbuf.split_off(cursor_idx);

                    // Create a new line and move the cursor to the next line.
                    self.buffers.insert(self.cursor.line + 1, nbuf);
                    self.cursor.index = 0;
                    self.cursor.line += 1;

                    self.renderer
                        .redraw(&Self::render_data(&self.buffers, &self.cursor))?;
                }
            }
            _ => { /* ignore */ }
        }
        Ok(true)
    }

    /// Delete the character before the cursor.
    fn delete_char_before_cursor(&mut self) -> usize {
        let idx = self.cursor.index;
        self.current_line_mut().remove(idx - 1);
        idx - 1
    }

    /// Insert the character before the cursor.
    fn insert_char_before_cursor(&mut self, c: char) -> usize {
        let idx = self.cursor.index;
        self.current_line_mut().insert(idx, c);
        idx + 1
    }

    // Returns an index that ensure that the cursor index is not overflowing the end.
    #[doc(hidden)]
    fn ensure_cursor_index(&self) -> usize {
        self.cursor.index.min(self.current_line_len())
    }
}

/// Builder struct for `MultilineTerm`.
#[derive(Default)]
pub struct MultilineTermBuilder<'w> {
    /// Initial buffer for the multiline terminal.
    buffers: Vec<String>,
    /// Initial line that the cursor is supposed to be set at.
    line: usize,
    /// Initial index that the cursor is supposed to be set at.
    index: usize,
    /// The renderer.
    renderer: Option<Box<dyn 'w + Renderer>>,
}

impl<'w> MultilineTermBuilder<'w> {
    /// Set the buffer that the terminal will be initialized with.
    #[inline]
    pub fn initial_buffers(mut self, buffers: Vec<String>) -> Self {
        self.buffers = buffers;
        self
    }

    /// Set what line the cursor will initially start at.
    #[inline]
    pub fn line(mut self, line: usize) -> Self {
        self.line = line;
        self
    }

    /// Set what index the cursor will initially start at.
    #[inline]
    pub fn index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn renderer<R: 'w + Renderer>(mut self, renderer: R) -> Self {
        self.renderer = Some(Box::new(renderer));
        self
    }

    /// Build a multiline terminal targeted to stdout.
    pub fn build_stdout(self) -> MultilineTerm<'w> {
        self.build_with_term()
    }

    /// Build a multiline terminal targeted to stderr.
    pub fn build_stderr(self) -> MultilineTerm<'w> {
        self.build_with_term()
    }

    fn build_with_term(self) -> MultilineTerm<'w> {
        MultilineTerm {
            buffers: self.buffers,
            cursor: Cursor {
                line: self.line,
                index: self.index,
            },
            renderer: self
                .renderer
                .unwrap_or_else(|| Box::new(LazyRenderer::default())),
        }
    }
}
