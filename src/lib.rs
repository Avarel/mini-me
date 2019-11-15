pub mod renderer;

use std::io;
use std::convert::TryInto;
use renderer::{Renderer, LazyRenderer};

pub use crossterm;

use crossterm::{
    input::{input, InputEvent, KeyEvent, MouseButton, MouseEvent},
    screen::RawScreen,
    Result,
};

/// Multiline abstraction around a terminal.
pub struct MultilineTerm {
    buffers: Vec<String>,
    cursor: Cursor,
    renderer: Box<dyn Renderer>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    /// Current line of the cursor.
    pub line: u16,
    /// Current index of the cursor.
    pub index: u16,
}

impl MultilineTerm {
    /// Create a builder for `MultilineTerm`.
    #[inline]
    pub fn builder() -> MultilineTermBuilder {
        MultilineTermBuilder::default()
    }

    /// Return the current buffer of the terminal.
    #[inline]
    pub fn buffers(&self) -> &Vec<String> {
        &self.buffers
    }

    /// Return a mutable reference to the buffer of the terminal.
    #[inline]
    pub fn buffers_mut(&mut self) -> &mut Vec<String> {
        &mut self.buffers
    }

    #[doc(hidden)]
    #[inline]
    fn current_line_len(&self) -> u16 {
        self.current_line().len().try_into().unwrap()
    }

    /// Return the cursor's position.
    #[inline]
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    /// Get a reference to current line of the cursor on the buffer.
    /// Unlike `current_line_mut`, this function will not allocate a new string
    /// if the buffer is empty, instead returning an empty string.
    pub fn current_line(&self) -> &str {
        if self.buffers.len() == 0 {
            return ""
        }
        &self.buffers[self.cursor.line as usize]
    }

    /// Get a mutable reference to the current line of the cursor on the buffer.
    /// 
    /// ### Warning
    /// This function will allocate a new `String` to the buffer if it is empty.
    pub fn current_line_mut(&mut self) -> &mut String {
        if self.buffers.is_empty() {
            let s = String::new();
            self.buffers.push(s);
            return &mut self.buffers[0]
        }
        let line = self.cursor.line;
        &mut self.buffers[line as usize]
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
        self.renderer.draw(&self)?;

        let _raw = RawScreen::into_raw_mode()?;
        let input = input();
        let mut sync_stdin = input.read_sync();

        loop {
            let event = sync_stdin.next();

            if let Some(key_event) = event {
                match key_event {
                    InputEvent::Keyboard(k) => if self.process_key_event(k)? { continue } else { break },
                    _ => {}
                }
            }
        }

        // Clear the last empty useless line.
        self.renderer.clear_line()?;
        self.renderer.flush()?;

        // If empty buffer, then return empty string.
        if self.buffers.is_empty() {
            return Ok(String::new())
        }

        // Join the buffers together, putting a `\n` in between each line.
        // Ensure the capacity to avoid reallocations.
        let mut buf = String::with_capacity(self.buffers.iter().map(|x| x.len() + 1).sum::<usize>() - 1);
        buf.push_str(&self.buffers[0]);
        for s in &self.buffers[1..] {
            buf.push('\n');
            buf.push_str(&s);
        }
        Ok(buf)
    }

    /// Return Ok(continue?)
    fn process_key_event(&mut self, event: KeyEvent) -> Result<bool> {
        match event {
            KeyEvent::Down => {
                if self.buffers.is_empty() { return Ok(true) }
                if self.cursor.line + 1 < self.buffers.len().try_into().unwrap() {
                    self.cursor.line += 1;
                    self.renderer.redraw(&self)?;
                }
            }
            KeyEvent::Up => {
                if self.buffers.is_empty() { return Ok(true) }
                if self.cursor.line > 0 {
                    self.cursor.line -= 1;
                    self.renderer.redraw(&self)?;
                }
            }
            KeyEvent::Left => {
                if self.buffers.is_empty() { return Ok(true) }
                self.cursor.index = self.ensure_cursor_index();
                if self.cursor.index > 0 {
                    self.cursor.index -= 1;
                } else if self.cursor.line > 0 {
                    // Move to the end of the previous line.
                    self.cursor.line -= 1;
                    self.cursor.index = self.buffers[self.cursor.line as usize].len().try_into().unwrap();
                }
                self.renderer.redraw(&self)?;
            }
            KeyEvent::Right => {
                if self.buffers.is_empty() { return Ok(true) }
                self.cursor.index = self.ensure_cursor_index();
                let len = self.current_line().len().try_into().unwrap();
                if self.cursor.index < len {
                    self.cursor.index += 1;
                } else if self.cursor.line + 1 < self.buffers.len().try_into().unwrap() {
                    // Move to the beginning of the next line.
                    self.cursor.line += 1;
                    self.cursor.index = 0;
                }
                self.renderer.redraw(&self)?;
            }
            KeyEvent::Backspace => {
                if self.buffers.is_empty() { return Ok(true) }
                self.cursor.index = self.ensure_cursor_index();

                if self.cursor.index > 0 {
                    self.cursor.index = self.delete_char_before_cursor();
                    self.renderer.redraw(&self)?;
                } else if self.cursor.line > 0 {
                    // Backspace at the beginning of the line, so push the contents of
                    // the current line to the line above it, and remove the line.

                    // Push the content of the current line to the previous line.
                    let cbuf = self.buffers.remove(self.cursor.line.try_into().unwrap());
                    // Change line number.
                    self.cursor.line -= 1;

                    // The cursor should now be at the end of the previous line
                    // before appending the contents of the current line.
                    self.cursor.index = self.current_line_len();
                    self.current_line_mut().push_str(&cbuf);
                
                    self.renderer.redraw(&self)?;
                }
            }
            KeyEvent::Char('p') => {
                self.renderer.clear_draw()?;
                self.renderer.flush()?;
            }
            KeyEvent::Char('[') => {
                self.renderer.clear_line()?;
                self.renderer.flush()?;
            }
            KeyEvent::Char(c) => {
                self.cursor.index = self.ensure_cursor_index();
                self.cursor.index = self.insert_char_before_cursor(c);
                self.renderer.redraw(&self)?;
            }
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
            KeyEvent::Enter => {
                if self.buffers.len() == 0 {
                    return Ok(false)
                } else if self.cursor.line + 1 == self.buffers.len().try_into().unwrap() && self.current_line_len() == 0 {
                    // Enter on the last line of the prompt which is also empty
                    // finishes the input.

                    // Remove last useless line.
                    self.buffers.remove((self.cursor.line).try_into().unwrap());
                    return Ok(false)
                } else {
                    self.cursor.index = self.ensure_cursor_index();
                    // Split the input after the cursor.
                    let cursor_idx = self.cursor.index;
                    let cbuf = self.current_line_mut();
                    let nbuf = cbuf.split_off((cursor_idx).try_into().unwrap());

                    // Create a new line and move the cursor to the next line.
                    self.buffers.insert((self.cursor.line + 1).try_into().unwrap(), nbuf);
                    self.cursor.index = 0;
                    self.cursor.line += 1;

                    self.renderer.redraw(&self)?;
                }
            }
            _ => { /* ignore */ }
        }
        Ok(true)
    }

    /// Delete the character before the cursor.
    fn delete_char_before_cursor(&mut self) -> u16 {
        let idx = self.cursor.index;
        self.current_line_mut().remove((idx - 1).try_into().unwrap());
        idx - 1
    }

    /// Insert the character before the cursor.
    fn insert_char_before_cursor(&mut self, c: char) -> u16 {
        let idx = self.cursor.index;
        let buf = self.current_line_mut();
        buf.insert(idx.try_into().unwrap(), c);
        idx + 1
    }

    // Returns an index that ensure that the cursor index is not overflowing the end.
    #[doc(hidden)]
    fn ensure_cursor_index(&self) -> u16 {
        self.cursor.index.min(self.current_line_len())
    }
}

/// Builder struct for `MultilineTerm`.
#[derive(Default)]
pub struct MultilineTermBuilder {
    /// Initial buffer for the multiline terminal.
    buffers: Vec<String>,
    /// Initial line that the cursor is supposed to be set at.
    line: u16,
    /// Initial index that the cursor is supposed to be set at.
    index: u16,
    /// The renderer.
    renderer: Option<Box<dyn Renderer>>,
}

impl MultilineTermBuilder {
    /// Set the buffer that the terminal will be initialized with.
    #[inline]
    pub fn initial_buffers(mut self, buffers: Vec<String>) -> Self {
        self.buffers = buffers;
        self
    }

    /// Set what line the cursor will initially start at.
    #[inline]
    pub fn line(mut self, line: u16) -> Self {
        self.line = line;
        self
    }

    /// Set what index the cursor will initially start at.
    #[inline]
    pub fn index(mut self, index: u16) -> Self {
        self.index = index;
        self
    }

    pub fn renderer<R: 'static + Renderer>(mut self, renderer: R) -> Self {
        self.renderer = Some(Box::new(renderer));
        self
    }

    /// Build a multiline terminal targeted to stdout.
    pub fn build_stdout(self) -> MultilineTerm {
        self.build_with_term()
    }

    /// Build a multiline terminal targeted to stderr.
    pub fn build_stderr(self) -> MultilineTerm {
        self.build_with_term()
    }

    fn build_with_term(self) -> MultilineTerm {
        MultilineTerm {
            buffers: self.buffers,
            cursor: Cursor {
                line: self.line,
                index: self.index,
            },
            renderer: self.renderer.unwrap_or_else(|| Box::new(LazyRenderer::default())),
        }
    }
}

