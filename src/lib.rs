use std::io;
use console::{Term, Key};

/// Reexport the console crate.
pub use console;

/// Multiline abstraction around a terminal.
///
/// This is a wrapper for the `Term` struct in the `console` crate.
pub struct MultilineTerm {
    /// Internal `console::term::Term` that this struct wraps around.
    inner: Term,
    buffers: Vec<String>,
    cursor: Cursor,
    options: MultilineTermOptions,
}

pub struct Cursor {
    /// Current line of the cursor.
    pub line: usize,
    /// Current index of the cursor.
    pub index: usize,
}

pub struct MultilineTermOptions {
    /// The mode of anchoring for the multiline terminal.
    anchor: AnchorMode,
    render: RenderMode,
    /// Helper field for anchor mode `Bottom`.
    empty_padding: usize,
    /// Buffer for each line in the multiline terminal.
    /// The buffer does not get allocated until the first letter has been typed.
    /// Function to draw the prompt.
    gutter: Option<Box<dyn Fn(usize, &MultilineTerm) -> String>>,
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
    fn current_line_len(&self) -> usize {
        self.current_line().len()
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
            return &mut self.buffers[0]
        }
        let line = self.cursor.line;
        &mut self.buffers[line]
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
        self.options.render.draw(&self)?;
        loop {
            match self.inner.read_key()? {
                Key::ArrowDown => {
                    if self.buffers.is_empty() { continue }
                    if self.cursor.line + 1 < self.buffers.len() {
                        self.cursor.line = self.move_cursor_down(1)?;
                        self.options.render.redraw(&self, RenderMode::Lazy)?;
                    }
                }
                Key::ArrowUp => {
                    if self.buffers.is_empty() { continue }
                    if self.cursor.line > 0 {
                        self.cursor.line = self.move_cursor_up(1)?;
                        self.options.render.redraw(&self, RenderMode::Lazy)?;
                    }
                }
                Key::ArrowLeft => {
                    if self.buffers.is_empty() { continue }
                    self.cursor.index = self.ensure_cursor_index();
                    if self.cursor.index > 0 {
                        self.cursor.index = self.move_cursor_left(1)?;
                    } else if self.cursor.line > 0 {
                        // Move to the end of the previous line.
                        self.cursor.line = self.move_cursor_up(1)?;
                        self.cursor.index = self.move_cursor_to_end()?;
                    }
                }
                Key::ArrowRight => {
                    if self.buffers.is_empty() { continue }
                    self.cursor.index = self.ensure_cursor_index();
                    let len = self.current_line().len();
                    if self.cursor.index < len {
                        self.cursor.index = self.move_cursor_right(1)?;
                    } else if self.cursor.line + 1 < self.buffers.len() {
                        // Move to the beginning of the next line.
                        self.cursor.line = self.move_cursor_down(1)?;
                        self.cursor.index = self.move_cursor_to_start()?;
                    }
                }
                Key::Back => {
                    if self.buffers.is_empty() { continue }
                    self.cursor.index = self.ensure_cursor_index();

                    if self.cursor.index > 0 {
                        self.cursor.index = self.delete_char_before_cursor();
                        self.options.render.redraw(&self, RenderMode::Lazy)?;
                    } else if self.cursor.line > 0 {
                        // Backspace at the beginning of the line, so push the contents of
                        // the current line to the line above it, and remove the line.
                        self.options.render.clear_draw(&self)?;

                        // Pad the top so the prompt can stay anchored to the bottom of the terminal.
                        if self.options.anchor == AnchorMode::Bottom {
                            self.options.empty_padding += 1;
                        }

                        // Push the content of the current line to the previous line.
                        let cbuf = self.buffers.remove(self.cursor.line);
                        // Change line number.
                        self.cursor.line -= 1;

                        // The cursor should now be at the end of the previous line
                        // before appending the contents of the current line.
                        self.cursor.index = self.current_line_len();
                        self.current_line_mut().push_str(&cbuf);
                    
                        self.options.render.draw(&self)?;
                    }
                }
                Key::Char(c) => {
                    self.cursor.index = self.ensure_cursor_index();
                    self.cursor.index = self.insert_char_before_cursor(c);
                    self.options.render.redraw(&self, RenderMode::Lazy)?;
                }
                Key::Escape => {
                    // Quick escape and finish the input.
                    if self.buffers.len() != 0 {
                        self.move_cursor_to_bottom()?;
                        if self.current_line_len() == 0 {
                            self.buffers.remove(self.cursor.line);
                        } else {
                            self.options.render.new_line(&self)?;
                        }
                    }
                    break
                }
                Key::Enter => {
                    if self.buffers.len() == 0 {
                        break
                    } else if self.cursor.line + 1 == self.buffers.len() && self.current_line_len() == 0 {
                        // Enter on the last line of the prompt which is also empty
                        // finishes the input.

                        // Remove last useless line.
                        self.buffers.remove(self.cursor.line);
                        break
                    } else {
                        self.cursor.index = self.ensure_cursor_index();
                        self.options.render.clear_draw(&self)?;
                        // Split the input after the cursor.
                        let cursor_idx = self.cursor.index;
                        let cbuf = self.current_line_mut();
                        let nbuf = cbuf.split_off(cursor_idx);

                        // Create a new line and move the cursor to the next line.
                        self.buffers.insert(self.cursor.line + 1, nbuf);
                        self.cursor.index = 0;
                        self.cursor.line += 1;

                        // Remove the padding if there is space to be taken up.
                        if self.options.anchor == AnchorMode::Bottom {
                            if self.options.empty_padding != 0 {
                                self.options.empty_padding -= 1;
                            }
                        }

                        self.options.render.draw(&self)?;
                    }
                }
                _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Unrecognized key input"))?
            };
        }

        // Clear the last empty useless line.
        self.inner.clear_line()?;

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
        let idx = self.cursor.index;
        self.current_line_mut().remove(idx - 1);
        idx - 1
    }

    /// Insert the character before the cursor.
    fn insert_char_before_cursor(&mut self, c: char) -> usize {
        let idx = self.cursor.index;
        let buf = self.current_line_mut();
        buf.insert(idx, c);
        idx + 1
    }

    /// Move the current cursor to the last line.
    #[inline]
    fn move_cursor_to_bottom(&self) -> io::Result<usize> {
        if self.buffers.len() == 0 { return Ok(0) }
        self.move_cursor_down(self.buffers.len() - self.cursor.line - 1)?;
        Ok(self.buffers.len())
    }

    /// Move the cursor to the end of the current line.
    /// This method is not safe to use if the cursor is not at `line:index`,
    #[inline]
    fn move_cursor_to_end(&self) -> io::Result<usize> {
        let len = self.current_line().len();
        if self.cursor.index > len {
            self.move_cursor_left(self.cursor.index - len)?;
        } else if self.cursor.index < len {
            self.move_cursor_right(len - self.cursor.index)?;
        }
        Ok(len)
    }

    /// Move the cursor to the beginning of the line.
    #[inline]
    fn move_cursor_to_start(&self) -> io::Result<usize> {
        self.move_cursor_left(self.cursor.index)?;
        Ok(0)
    }

    /// Move the cursor one line up.
    #[inline]
    fn move_cursor_up(&self, n: usize) -> io::Result<usize> {
        self.inner.move_cursor_up(n)?;
        if self.cursor.line == 0 {
            Ok(0)
        } else {
            Ok(self.cursor.line - n)
        }
    }

    /// Move the cursor one line down.
    #[inline]
    fn move_cursor_down(&self, n: usize) -> io::Result<usize> {
        self.inner.move_cursor_down(n)?;
        Ok(self.cursor.line + n)
    }

    /// Move the cursor leftward using nondestructive backspaces.
    #[inline]
    fn move_cursor_left(&self, n: usize) -> io::Result<usize> {
        self.inner.move_cursor_left(n)?;
        if self.cursor.index == 0 {
            Ok(0)
        } else {
            Ok(self.cursor.index - n)
        }
    }

    /// Move the cursor rightward.
    /// This method is not safe to use if the cursor is not at `line:index`,
    /// as it draws from the buffer to move forward.
    #[inline]
    fn move_cursor_right(&self, n: usize) -> io::Result<usize> {
        self.inner.move_cursor_right(n)?;
        Ok(self.cursor.index + n)
    }

    // Returns an index that ensure that the cursor index is not overflowing the end.
    #[doc(hidden)]
    fn ensure_cursor_index(&self) -> usize {
        self.cursor.index.min(self.current_line_len())
    }
}

/// Builder struct for `MultilineTerm`.
#[derive(Default)]
pub struct MultilineTermBuilder {
    /// The mode of anchoring for the multiline terminal.
    anchor: AnchorMode,
    /// Mode of rendering
    render: RenderMode,
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
    pub fn anchor(mut self, anchor: AnchorMode) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn render(mut self, render: RenderMode) -> Self {
        self.render = render;
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
            inner: Term::stdout(),
            buffers: self.buffers,
            cursor: Cursor {
                line: self.line,
                index: self.index,
            },
            options: MultilineTermOptions {
                anchor: self.anchor,
                render: self.render,
                empty_padding: 0,
                gutter: self.gutter
            }
        }
    }

    /// Build a multiline terminal targeted to stderr.
    pub fn build_stderr(self) -> MultilineTerm {
        MultilineTerm {
            inner: Term::stderr(),
            buffers: self.buffers,
            cursor: Cursor {
                line: self.line,
                index: self.index,
            },
            options: MultilineTermOptions {
                anchor: self.anchor,
                render: self.render,
                empty_padding: 0,
                gutter: self.gutter
            }
        }
    }
}

/// The mode of anchoring of the multiline prompt.
#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum AnchorMode {
    /// Default mode of anchoring for the multiline prompt.
    /// The multiline prompt will always be anchored where it is first printed.
    InPlace,
    /// Alternative mode of anchoring for the multiline prompt.
    /// The multiline prompt will always be anchored at the bottom of the terminal.
    Bottom,
}

impl Default for AnchorMode {
    fn default() -> Self {
        AnchorMode::InPlace
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderMode {
    /// Allows for proper prompt drawing everytime and
    /// terminal-wide visual updates to the input.
    Full,
    /// Most performant, but may be limited when trying
    /// to render more advanced options.
    Lazy,
}

impl Default for RenderMode {
    fn default() -> Self {
        RenderMode::Lazy
    }
}

impl RenderMode {
    /// Draw the prompt.
    fn draw(self, term: &MultilineTerm) -> io::Result<()> {
        // Handle empty buffer.
        if term.buffers.is_empty() {
            if let Some(f) = &term.options.gutter {
                term.inner.write_str(&f(0, term))?;
            }
            return Ok(())
        }

        if let AnchorMode::Bottom = term.options.anchor {
            // Print out the padding.
            for _ in 0..term.options.empty_padding {
                self.new_line(term)?;
            }
        }

        // Print out the contents.
        for i in 0..term.buffers.len() {
            self.draw_line(term, i)?;
            if i < term.buffers.len() - 1 {
                // The last line should not have any new-line attached to it.
                self.new_line(term)?;
            }
        }

        // Position the cursor.
        // At this point the cursor is pointed at the very end of the last line.
        let last_len = term.buffers.last().unwrap().len();
        term.move_cursor_up(term.buffers.len() - term.cursor.line - 1)?;
        if term.cursor.index < last_len {
            term.move_cursor_left(last_len - term.cursor.index)?;
        } else if term.cursor.index > last_len && term.cursor.line + 1 != term.buffers.len() {
            term.move_cursor_right(term.current_line_len().min(term.cursor.index) - last_len)?;
        }

        Ok(())
    }

    /// Clear the drawn prompt on the screen.
    fn clear_draw(self, term: &MultilineTerm) -> io::Result<()> {
        term.move_cursor_to_bottom()?;
        term.inner.clear_line()?;
        if term.buffers.len() != 0 {
            term.inner.clear_last_lines(term.buffers.len() - 1 + term.options.empty_padding)?;
        }
        Ok(())
    }

    /// Redraw the screen.
    fn redraw(self, term: &MultilineTerm, hint: RenderMode) -> io::Result<()> {
        if self == RenderMode::Lazy && hint == RenderMode::Lazy {
            self.clear_current_line(term)?;
            self.draw_current_line(term)
        } else {
            self.clear_draw(term)?;
            self.draw(term)
        }
    }

    /// Draw the line given an index.
    /// This method does not move the cursor.
    fn draw_line(self, term: &MultilineTerm, index: usize) -> io::Result<()> {
        if let Some(f) = &term.options.gutter {
            term.inner.write_str(&f(index, term))?;
        }
        term.inner.write_str(&term.buffers[index])
    }

    /// Draw the current line and move the cursor appropriately.
    fn draw_current_line(self, term: &MultilineTerm) -> io::Result<()> {
        self.draw_line(term, term.cursor.line)?;
        // disable this check if you want overflow cursor
        if term.cursor.index < term.current_line_len() {
            term.move_cursor_left(term.current_line_len() - term.cursor.index)?;
        }
        Ok(())
    }

    /// Clear the current line on the screen.
    #[inline]
    fn clear_current_line(self, term: &MultilineTerm) -> io::Result<()> {
        term.inner.clear_line()
    }

    /// Insert a new line on the screen.
    #[inline]
    fn new_line(self, term: &MultilineTerm) -> io::Result<()> {
        term.inner.write_line("")
    }
}