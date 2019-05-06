use std::io::{self};

use console::{Term, Key};

fn main() -> io::Result<()> {
    let mut prompt = MultilineTerm::stdout();
    prompt.prompt = Some(|i| format!("{:^3}| ", i));

    let result = prompt.read_multiline()?;
    println!("{}", result);
    Ok(())
}

struct MultilineTerm<F> {
    term: Term,
    empty_padding: usize,
    pub buffers: Vec<String>,
    pub line: usize,
    pub index: usize,
    pub prompt: Option<F>,
}

impl<F: Fn(usize) -> String> MultilineTerm<F> {
    /// Return a new unbuffered multiline terminal.
    pub fn stdout() -> Self {
        Self {
            term: Term::stdout(),
            buffers: Vec::new(),
            empty_padding: 0,
            line: 1,
            index: 0,
            prompt: None
        }
    }

    /// Return a new unbuffered multiline terminal to stderr.
    pub fn stderr() -> Self {
        Self {
            term: Term::stderr(),
            buffers: Vec::new(),
            empty_padding: 0,
            line: 1,
            index: 0,
            prompt: None
        }
    }

    #[doc(hidden)]
    fn current_line_len(&self) -> usize {
        self.current_line().len()
    }

    /// Get a reference to current line of the cursor on the buffer.
    pub fn current_line(&self) -> &str {
        &self.buffers[self.line - 1]
    }

    /// Get a mutable reference to the current line of the cursor on the buffer.
    pub fn current_line_mut(&mut self) -> &mut String {
        &mut self.buffers[self.line - 1]
    }

    /// Read multiple lines of input.
    /// 
    /// * `Enter` on an empty last line will submit the input.
    /// * `Enter` on a non-empty line will create a new line.
    /// * `Backspace` at the beginning of the line to tappend the content
    ///   of the current line to the previous line.
    /// 
    /// This does not include the empty line or trailing newline.
    pub fn read_multiline(mut self) -> io::Result<String> {
        self.draw()?;
        loop {
            match self.term.read_key()? {
                Key::ArrowDown => {
                    if self.buffers.is_empty() { continue }
                    if self.line < self.buffers.len() {
                        self.line = self.move_cursor_down(1)?;

                        // Ensure that the cursor isn't beyond the end of the line.
                        if self.index > self.current_line_len() {
                            self.index = self.move_cursor_to_end()?;
                        }
                    }
                }
                Key::ArrowUp => {
                    if self.buffers.is_empty() { continue }
                    if self.line > 1 {
                        self.line = self.move_cursor_up(1)?;

                        // Ensure that the cursor isn't beyond the end of the line.
                        if self.index > self.current_line_len() {
                            self.index = self.move_cursor_to_end()?;
                        }
                    }
                }
                Key::ArrowLeft => {
                    if self.buffers.is_empty() { continue }
                    if self.index > 0 {
                        self.index = self.move_cursor_left(1)?;
                    } else if self.line > 1 {
                        // Move to the end of the previous line.
                        self.line = self.move_cursor_up(1)?;
                        self.index = self.move_cursor_to_end()?;
                    }
                }
                Key::ArrowRight => {
                    if self.buffers.is_empty() { continue }
                    let len = self.current_line().len();
                    if self.index < len {
                        self.index = self.move_cursor_right(1)?;
                    } else if self.line < self.buffers.len() {
                        // Move to the beginning of the next line.
                        self.line = self.move_cursor_down(1)?;
                        self.index = self.move_cursor_to_start()?;
                    }
                }
                Key::Char('\x7f') => {
                    if self.buffers.is_empty() { continue }
                    if self.index > 0 {
                        self.index = self.delete_char_before_cursor();
                        self.redraw()?;
                    } else if self.line > 1 {
                        // Backspace at the beginning of the line, so push the contents of
                        // the current line to the line above it, and remove the line.
                        self.empty_padding += 1;
                        
                        // Push the content of the current line to the previous line.
                        let cbuf = self.buffers.remove(self.line - 1);
                        // Change line number.
                        self.line -= 1;

                        // The cursor should now be at the end of the previous line
                        // before appending the contents of the current line.
                        self.index = self.current_line_len();
                        self.current_line_mut().push_str(&cbuf);
                    
                        self.redraw()?;
                    }
                }
                Key::Char(c) => {
                    self.index = self.insert_char_before_cursor(c);
                    self.redraw_current_line()?;
                }
                Key::Escape => {
                    // Quick escape and finish the input.
                    if self.buffers.len() != 0 {
                        self.move_cursor_to_bottom()?;
                        if self.current_line_len() == 0 {
                            self.buffers.remove(self.line - 1);
                        } else {
                            self.new_line()?;
                        }
                    }
                    break
                }
                Key::Enter => {
                    if self.buffers.len() == 0 {
                        break
                    } else if self.line == self.buffers.len() && self.current_line_len() == 0 {
                        // Enter on the last line of the prompt which is also empty
                        // finishes the input.

                        // Remove last useless line.
                        self.buffers.remove(self.line - 1);
                        break
                    } else {
                        // Split the input after the cursor.
                        let cursor_idx = self.index;
                        let cbuf = self.current_line_mut();
                        let nbuf = cbuf.split_off(cursor_idx);

                        // Create a new line and move the cursor to the next line.
                        self.buffers.insert(self.line, nbuf);
                        self.index = 0;
                        self.line += 1;

                        if self.empty_padding == 0 {
                            // If theres no padding to take up:
                            // Move the cursor to the bottom in order to force a new line be
                            // printed so that the redraw don't draw over any other input.
                            self.move_cursor_to_bottom()?;
                            self.new_line()?;
                        } else {
                            // The padding is created whenever there's an extra line
                            // created by backspacing at the beginning of a line and deleting it.
                            // The padding is so that the typing experience still flows nicely.
                            // If there is already padding, then the new line will just take it up
                            // instead of creating another line
                            self.empty_padding -= 1;
                        }

                        self.redraw()?;
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
        if self.buffers.is_empty() {
            self.buffers.push(String::new());
        }

        let idx = self.index;
        let buf = self.current_line_mut();
        buf.insert(idx, c);
        idx + 1
    }

    /// Draw the prompt.
    fn draw(&self) -> io::Result<()> {
        // Handle empty buffer.
        if self.buffers.is_empty() {
            if let Some(f) = &self.prompt {
                self.term.write_str(&f(0))?;
            }
            return Ok(())
        }

        // Print out the padding.
        for _ in 0..self.empty_padding {
            self.new_line()?;
        }

        // Print out the contents.
        for i in 0..self.buffers.len() {
            self.draw_line(i + 1)?;
            if i < self.buffers.len() - 1 {
                // The last line should not have any new-line attached to it.
                self.new_line()?;
            }
        }
        
        // Position the cursor.
        // At this point the cursor is pointed at the very end of the last line.
        let last_len = self.buffers.last().unwrap().len();
        self.move_cursor_up(self.buffers.len() - self.line)?;
        // self.move_cursor_left(last_len)?;
        // self.move_cursor_right(self.index)?;
        if self.index < last_len {
            self.move_cursor_left(last_len - self.index)?;
        } else if self.index > last_len {
            // Not safe to use move_cursor_right because that method assumes
            // that the cursor is exactly at `line:index`, which is not what
            // the drawn cursor is at.
            self.term.write_str(&self.current_line()[last_len..self.index])?;
        }

        Ok(())
    }

    /// Clear the drawn prompt on the screen.
    fn clear_draw(&self) -> io::Result<()> {
        self.move_cursor_to_bottom()?;
        self.term.clear_line()?;
        self.term.clear_last_lines(self.buffers.len() - 1 + self.empty_padding)?;
        Ok(())
    }

    /// Redraw the screen.
    fn redraw(&self) -> io::Result<()> {
        self.clear_draw()?;
        self.draw()
    }

    /// Draw the line (`index` starts at 1).
    fn draw_line(&self, index: usize) -> io::Result<()> {
        let i = index - 1;
        if let Some(f) = &self.prompt {
            self.term.write_str(&f(i))?;
        }
        self.term.write_str(&self.buffers[i])
    }

    // Draw the current line.
    fn draw_current_line(&self) -> io::Result<()> {
        self.draw_line(self.line)?;
        self.move_cursor_left(self.current_line_len() - self.index)?;
        Ok(())
    }

    /// Clear the current line on the screen.
    fn clear_current_line(&self) -> io::Result<()> {
        self.term.clear_line()
    }

    // Redraw the current line, which is cheaper than clearing and redrawing the entire line.
    fn redraw_current_line(&self) -> io::Result<()> {
        self.clear_current_line()?;
        self.draw_current_line()
    }

    /// Insert a new line on the screen.
    fn new_line(&self) -> io::Result<()> {
        self.term.write_line("")
    }

    /// Move the current cursor to the last line.
    fn move_cursor_to_bottom(&self) -> io::Result<usize> {
        if self.buffers.len() == 0 { return Ok(0) }
        self.term.move_cursor_down(self.buffers.len() - self.line + 1)?;
        Ok(self.buffers.len())
    }

    /// Move the cursor one line up.
    fn move_cursor_up(&self, n: usize) -> io::Result<usize> {
        self.term.move_cursor_up(n)?;
        Ok(self.line - 1)
    }

    /// Move the cursor one line down.
    fn move_cursor_down(&self, n: usize) -> io::Result<usize> {
        self.term.move_cursor_down(n)?;
        Ok(self.line + 1)
    }

    /// Move the cursor to the end of the current line.
    /// This method is not safe to use if the cursor is not at `line:index`,
    /// as it may draw from the buffer to move forward if the cursor needs
    /// to move in the rightward direction.
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
    fn move_cursor_to_start(&self) -> io::Result<usize> {
        self.move_cursor_left(self.index)?;
        Ok(0)
    }

    /// Move the cursor leftward using nondestructive backspaces.
    fn move_cursor_left(&self, n: usize) -> io::Result<usize> {
        for _ in 0..n {
            self.term.write_str("\x08")?;
        }
        if self.index == 0 {
            Ok(0)
        } else {
            Ok(self.index - 1)
        }
    }

    /// Move the cursor rightward.
    /// This method is not safe to use if the cursor is not at `line:index`,
    /// as it draws from the buffer to move forward.
    fn move_cursor_right(&self, n: usize) -> io::Result<usize> {
        let cbuf = self.current_line();
        self.term.write_str(&cbuf[self.index..self.index + n])?;
        Ok(self.index + 1)
    }
}