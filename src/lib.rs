use std::io;
use console::{Term, Key};

/// Reexport the console crate.
pub use console;

/// Multiline abstraction around a terminal.
///
/// This is a wrapper for the `Term` struct in the `console` crate.
pub struct MultilineTerm {
    /// Internal `console::term::Term` that this struct wraps around.
    term: Term,
    /// The mode of anchoring for the multiline terminal.
    anchor: Anchor,
    /// Helper field for anchor mode `Bottom`.
    empty_padding: usize,
    /// Buffer for each line in the multiline terminal.
    /// The buffer does not get allocated until the first letter has been typed.
    buffers: Vec<String>,
    /// Current line of the cursor.
    line: usize,
    /// Current index of the cursor.
    index: usize,
    /// Function to draw the prompt.
    gutter: Option<Box<dyn Fn(usize, &Self) -> String>>,
}

// TODO: separate certain properties into `MultilineTermOptions`

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
    fn current_line_len(&self) -> usize {
        self.current_line().len()
    }

    /// Return the cursor's line number.
    #[inline]
    pub fn cursor_line(&self) -> usize {
        self.line
    }

    /// Return the cursor's index number.
    #[inline]
    pub fn cursor_index(&self) -> usize {
        self.index
    }

    /// Get a reference to current line of the cursor on the buffer.
    /// Unlike `current_line_mut`, this function will not allocate a new string
    /// if the buffer is empty, instead returning an empty string.
    pub fn current_line(&self) -> &str {
        if self.buffers().len() == 0 {
            return ""
        }
        &self.buffers()[self.line]
    }

    /// Get a mutable reference to the current line of the cursor on the buffer.
    /// 
    /// ### Warning
    /// This function will allocate a new `String` to the buffer if it is empty.
    pub fn current_line_mut(&mut self) -> &mut String {
        if self.buffers().is_empty() {
            let s = String::new();
            self.buffers_mut().push(s);
            return &mut self.buffers_mut()[0]
        }
        let line = self.line;
        &mut self.buffers_mut()[line]
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
    pub fn read_multiline(mut self) -> io::Result<String> {
        self.draw()?;
        loop {
            match self.term.read_key()? {
                Key::ArrowDown => {
                    if self.buffers.is_empty() { continue }
                    if self.line + 1 < self.buffers.len() {
                        self.line = self.move_cursor_down(1)?;
                        self.redraw_current_line()?;
                    }
                }
                Key::ArrowUp => {
                    if self.buffers.is_empty() { continue }
                    if self.line > 0 {
                        self.line = self.move_cursor_up(1)?;
                        self.redraw_current_line()?;
                    }
                }
                Key::ArrowLeft => {
                    if self.buffers.is_empty() { continue }
                    self.index = self.ensure_cursor_index();
                    if self.index > 0 {
                        self.index = self.move_cursor_left(1)?;
                    } else if self.line > 0 {
                        // Move to the end of the previous line.
                        self.line = self.move_cursor_up(1)?;
                        self.index = self.move_cursor_to_end()?;
                    }
                }
                Key::ArrowRight => {
                    if self.buffers.is_empty() { continue }
                    self.index = self.ensure_cursor_index();
                    let len = self.current_line().len();
                    if self.index < len {
                        self.index = self.move_cursor_right(1)?;
                    } else if self.line + 1 < self.buffers.len() {
                        // Move to the beginning of the next line.
                        self.line = self.move_cursor_down(1)?;
                        self.index = self.move_cursor_to_start()?;
                    }
                }
                Key::Back => {
                    if self.buffers.is_empty() { continue }
                    self.index = self.ensure_cursor_index();

                    if self.index > 0 {
                        self.index = self.delete_char_before_cursor();
                        self.redraw_current_line()?;
                    } else if self.line > 0 {
                        // Backspace at the beginning of the line, so push the contents of
                        // the current line to the line above it, and remove the line.
                        self.clear_draw()?;

                        // Pad the top so the prompt can stay anchored to the bottom of the terminal.
                        if let Anchor::Bottom = self.anchor {
                            self.empty_padding += 1;
                        }

                        // Push the content of the current line to the previous line.
                        let cbuf = self.buffers.remove(self.line);
                        // Change line number.
                        self.line -= 1;

                        // The cursor should now be at the end of the previous line
                        // before appending the contents of the current line.
                        self.index = self.current_line_len();
                        self.current_line_mut().push_str(&cbuf);
                    
                        self.draw()?;
                    }
                }
                Key::Char(c) => {
                    self.index = self.ensure_cursor_index();
                    self.index = self.insert_char_before_cursor(c);
                    self.redraw_current_line()?;
                }
                Key::Escape => {
                    // Quick escape and finish the input.
                    if self.buffers.len() != 0 {
                        self.move_cursor_to_bottom()?;
                        if self.current_line_len() == 0 {
                            self.buffers.remove(self.line);
                        } else {
                            self.new_line()?;
                        }
                    }
                    break
                }
                Key::Enter => {
                    if self.buffers.len() == 0 {
                        break
                    } else if self.line + 1 == self.buffers.len() && self.current_line_len() == 0 {
                        // Enter on the last line of the prompt which is also empty
                        // finishes the input.

                        // Remove last useless line.
                        self.buffers.remove(self.line);
                        break
                    } else {
                        self.index = self.ensure_cursor_index();
                        self.clear_draw()?;
                        // Split the input after the cursor.
                        let cursor_idx = self.index;
                        let cbuf = self.current_line_mut();
                        let nbuf = cbuf.split_off(cursor_idx);

                        // Create a new line and move the cursor to the next line.
                        self.buffers.insert(self.line + 1, nbuf);
                        self.index = 0;
                        self.line += 1;

                        // Remove the padding if there is space to be taken up.
                        if let Anchor::Bottom = self.anchor {
                            if self.empty_padding != 0 {
                                self.empty_padding -= 1;
                            }
                        }

                        self.draw()?;
                    }
                }
                _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Unrecognized key input"))?
            };
        }

        // Clear the last empty useless line.
        self.term.clear_line()?;

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

    /// Delete the character before the cursor.
    fn delete_char_before_cursor(&mut self) -> usize {
        let idx = self.index;
        self.current_line_mut().remove(idx - 1);
        idx - 1
    }

    /// Insert the character before the cursor.
    fn insert_char_before_cursor(&mut self, c: char) -> usize {
        let idx = self.index;
        let buf = self.current_line_mut();
        buf.insert(idx, c);
        idx + 1
    }

    /// Draw the prompt.
    fn draw(&self) -> io::Result<()> {
        // Handle empty buffer.
        if self.buffers.is_empty() {
            if let Some(f) = &self.gutter {
                self.term.write_str(&f(0, self))?;
            }
            return Ok(())
        }

        if let Anchor::Bottom = self.anchor {
            // Print out the padding.
            for _ in 0..self.empty_padding {
                self.new_line()?;
            }
        }

        // Print out the contents.
        for i in 0..self.buffers.len() {
            self.draw_line(i)?;
            if i < self.buffers.len() - 1 {
                // The last line should not have any new-line attached to it.
                self.new_line()?;
            }
        }

        // Position the cursor.
        // At this point the cursor is pointed at the very end of the last line.
        let last_len = self.buffers.last().unwrap().len();
        self.move_cursor_up(self.buffers.len() - self.line - 1)?;
        if self.index < last_len {
            self.move_cursor_left(last_len - self.index)?;
        } 

        Ok(())
    }

    /// Clear the drawn prompt on the screen.
    fn clear_draw(&self) -> io::Result<()> {
        self.move_cursor_to_bottom()?;
        self.term.clear_line()?;
        if self.buffers.len() != 0 {
            self.term.clear_last_lines(self.buffers.len() - 1 + self.empty_padding)?;
        }
        Ok(())
    }

    /// Redraw the screen.
    fn redraw(&self) -> io::Result<()> {
        self.clear_draw()?;
        self.draw()
    }

    /// Draw the line given an index.
    /// This method does not move the cursor.
    fn draw_line(&self, index: usize) -> io::Result<()> {
        if let Some(f) = &self.gutter {
            self.term.write_str(&f(index, self))?;
        }
        self.term.write_str(&self.buffers[index])
    }

    /// Draw the current line and move the cursor appropriately.
    fn draw_current_line(&self) -> io::Result<()> {
        self.draw_line(self.line)?;
        // disable this check if you want overflow cursor
        if  self.index < self.current_line_len() {
            self.move_cursor_left(self.current_line_len() - self.index)?;
        }
        Ok(())
    }

    /// Clear the current line on the screen.
    #[inline]
    fn clear_current_line(&self) -> io::Result<()> {
        self.term.clear_line()
    }

    /// Redraw the current line, which is cheaper than clearing and redrawing the entire line.
    #[inline]
    fn redraw_current_line(&self) -> io::Result<()> {
        self.clear_current_line()?;
        self.draw_current_line()
    }

    /// Insert a new line on the screen.
    #[inline]
    fn new_line(&self) -> io::Result<()> {
        self.term.write_line("")
    }

    /// Move the current cursor to the last line.
    #[inline]
    fn move_cursor_to_bottom(&self) -> io::Result<usize> {
        if self.buffers.len() == 0 { return Ok(0) }
        self.term.move_cursor_down(self.buffers.len() - self.line - 1)?;
        Ok(self.buffers.len())
    }

    /// Move the cursor to the end of the current line.
    /// This method is not safe to use if the cursor is not at `line:index`,
    #[inline]
    fn move_cursor_to_end(&self) -> io::Result<usize> {
        let len = self.current_line().len();
        if self.index > len {
            self.move_cursor_left(self.index - len)?;
        } else if self.index < len {
            self.move_cursor_right(len - self.index)?;
        }
        Ok(len)
    }

    /// Move the cursor to the beginning of the line.
    #[inline]
    fn move_cursor_to_start(&self) -> io::Result<usize> {
        self.move_cursor_left(self.index)?;
        Ok(0)
    }

    /// Move the cursor one line up.
    #[inline]
    fn move_cursor_up(&self, n: usize) -> io::Result<usize> {
        self.term.move_cursor_up(n)?;
        if self.line == 0 {
            Ok(0)
        } else {
            Ok(self.line - 1)
        }
    }

    /// Move the cursor one line down.
    #[inline]
    fn move_cursor_down(&self, n: usize) -> io::Result<usize> {
        self.term.move_cursor_down(n)?;
        Ok(self.line + 1)
    }

    /// Move the cursor leftward using nondestructive backspaces.
    #[inline]
    fn move_cursor_left(&self, n: usize) -> io::Result<usize> {
        self.term.move_cursor_left(n)?;
        if self.index == 0 {
            Ok(0)
        } else {
            Ok(self.index - 1)
        }
    }

    /// Move the cursor rightward.
    /// This method is not safe to use if the cursor is not at `line:index`,
    /// as it draws from the buffer to move forward.
    #[inline]
    fn move_cursor_right(&self, n: usize) -> io::Result<usize> {
        self.term.move_cursor_right(n)?;
        Ok(self.index + 1)
    }

    // Returns an index that ensure that the cursor index is not overflowing the end.
    #[doc(hidden)]
    fn ensure_cursor_index(&self) -> usize {
        self.index.min(self.current_line_len())
    }
}

/// Builder struct for `MultilineTerm`.
#[derive(Default)]
pub struct MultilineTermBuilder {
    /// The mode of anchoring for the multiline terminal.
    anchor: Anchor,
    /// Initial buffer for the multiline terminal.
    buffers: Vec<String>,
    /// Initial line that the cursor is supposed to be set at.
    line: usize,
    /// Initial index that the cursor is supposed to be set at.
    index: usize,
    /// Function to draw the gutter.
    gutter: Option<Box<dyn Fn(usize, &MultilineTerm) -> String>>,
}

impl MultilineTermBuilder {
    /// Sets the anchor mode for the multiline terminal, 
    /// which can either be `InPlace` or `Bottom`.
    #[inline]
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

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

    /// Set the function that provides the prompt printing.
    #[inline]
    pub fn gutter<F: 'static + Fn(usize, &MultilineTerm) -> String>(mut self, f: F)  -> Self {
        self.gutter = Some(Box::new(f));
        self
    }

    /// Build a multiline terminal targeted to stdout.
    pub fn build_stdout(self) -> MultilineTerm {
        MultilineTerm {
            term: Term::stdout(),
            anchor: self.anchor,
            buffers: self.buffers,
            line: self.line,
            index: self.index,
            empty_padding: 0,
            gutter: self.gutter
        }
    }

    /// Build a multiline terminal targeted to stderr.
    pub fn build_stderr(self) -> MultilineTerm {
        MultilineTerm {
            term: Term::stderr(),
            anchor: self.anchor,
            buffers: self.buffers,
            line: self.line,
            index: self.index,
            empty_padding: 0,
            gutter: self.gutter
        }
    }
}

/// The mode of anchoring of the multiline prompt.
#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Anchor {
    /// Default mode of anchoring for the multiline prompt.
    /// The multiline prompt will always be anchored where it is first printed.
    InPlace,
    /// Alternative mode of anchoring for the multiline prompt.
    /// The multiline prompt will always be anchored at the bottom of the terminal.
    Bottom,
}

impl Default for Anchor {
    fn default() -> Self {
        Anchor::InPlace
    }
}