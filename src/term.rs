use std::io;
use console::{Term, Key};
use crate::console_patch::*;

pub struct MultilineTerm {
    term: Term,
    empty_padding: usize,
    anchor: Anchor,
    buffers: Vec<String>,
    line: usize,
    index: usize,
    prompt: Option<fn(usize, usize, &str) -> String>,
}

impl MultilineTerm {
    pub fn builder() -> MultilineTermBuilder {
        MultilineTermBuilder::default()
    }

    #[doc(hidden)]
    fn current_line_len(&self) -> usize {
        self.current_line().len()
    }

    /// Get a reference to current line of the cursor on the buffer.
    pub fn current_line(&self) -> &str {
        &self.buffers[self.line]
    }

    /// Get a mutable reference to the current line of the cursor on the buffer.
    pub fn current_line_mut(&mut self) -> &mut String {
        &mut self.buffers[self.line]
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
                    if self.line + 1 < self.buffers.len() {
                        self.line = self.move_cursor_down(1)?;

                        // Ensure that the cursor isn't beyond the end of the line.
                        if self.index > self.current_line_len() {
                            self.index = self.move_cursor_to_end()?;
                        }
                    }
                }
                Key::ArrowUp => {
                    if self.buffers.is_empty() { continue }
                    if self.line > 0 {
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
                    } else if self.line > 0 {
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
                Key::Char('\x7f') /* unix */ | Key::Char('\x08') /* windows */ => {
                    if self.buffers.is_empty() { continue }
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
                self.term.write_str(&f(0, 1, ""))?;
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
        } else if self.index > last_len {
            self.move_cursor_right(self.index)?;
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
    fn draw_line(&self, index: usize) -> io::Result<()> {
        if let Some(f) = &self.prompt {
            self.term.write_str(&f(index, self.buffers.len(), &self.buffers[index]))?;
        }
        self.term.write_str(&self.buffers[index])
    }

    /// Draw the current line.
    fn draw_current_line(&self) -> io::Result<()> {
        self.draw_line(self.line)?;
        self.move_cursor_left(self.current_line_len() - self.index)?;
        Ok(())
    }

    /// Clear the current line on the screen.
    fn clear_current_line(&self) -> io::Result<()> {
        self.term.clear_line()
    }

    /// Redraw the current line, which is cheaper than clearing and redrawing the entire line.
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
        move_cursor_down(&self.term, self.buffers.len() - self.line - 1)?;
        Ok(self.buffers.len())
    }

    /// Move the cursor one line up.
    fn move_cursor_up(&self, n: usize) -> io::Result<usize> {
        move_cursor_up(&self.term, n)?;
        if self.line == 0 {
            Ok(0)
        } else {
            Ok(self.line - 1)
        }
    }

    /// Move the cursor one line down.
    fn move_cursor_down(&self, n: usize) -> io::Result<usize> {
        move_cursor_down(&self.term, n)?;
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
        move_cursor_left(&self.term, n)?;
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
        move_cursor_right(&self.term, n)?;
        Ok(self.index + 1)
    }
}

#[derive(Default)]
pub struct MultilineTermBuilder {
    anchor: Anchor,
    buffers: Vec<String>,
    line: usize,
    index: usize,
    prompt: Option<fn(usize, usize, &str) -> String>,
}

#[allow(dead_code)]
impl MultilineTermBuilder {
    pub fn anchor(&mut self, anchor: Anchor) -> &mut Self {
        self.anchor = anchor;
        self
    }

    pub fn buffers(&mut self, buffers: Vec<String>) -> &mut Self {
        self.buffers = buffers;
        self
    }

    pub fn line(&mut self, line: usize) -> &mut Self {
        self.line = line;
        self
    }

    pub fn index(&mut self, index: usize) -> &mut Self {
        self.index = index;
        self
    }

    pub fn prompt(&mut self, f: fn(usize, usize, &str) -> String) -> &mut Self {
        self.prompt = Some(f);
        self
    }

    pub fn build_stdout(&mut self) -> MultilineTerm {
        MultilineTerm {
            term: Term::stdout(),
            anchor: self.anchor,
            buffers: self.buffers.clone(),
            line: self.line,
            index: self.index,
            empty_padding: 0,
            prompt: self.prompt.clone()
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