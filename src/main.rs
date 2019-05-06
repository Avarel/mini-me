use std::io::{self};

use console::{Term, Key};

fn main() -> io::Result<()> {
    let mut prompt = MultilineTerm::stdout();
    prompt.buffers = vec![
        "hello there".to_owned(),
        "howtheheck are you doing".to_owned(),
        "memes".to_owned()
    ];
    prompt.line = 2;
    prompt.index = 3;
    prompt.prompt = Some(("  in > ".to_owned(), " ... > ".to_owned()));

    let result = prompt.read_prompt()?;
    println!("{}", result);
    Ok(())
}

struct MultilineTerm {
    term: Term,
    empty_padding: usize,
    pub buffers: Vec<String>,
    pub line: usize,
    pub index: usize,
    pub prompt: Option<(String, String)>,
}

impl MultilineTerm {
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

    fn current_line_len(&self) -> usize {
        self.current_line().len()
    }

    fn current_line(&self) -> &String {
        &self.buffers[self.line - 1]
    }

    fn current_line_mut(&mut self) -> &mut String {
        &mut self.buffers[self.line - 1]
    }

    pub fn read_prompt(mut self) -> io::Result<String> {
        self.draw()?;
        loop {
            match self.term.read_key()? {
                Key::ArrowDown => {
                    if self.line < self.buffers.len() {
                        self.line = self.move_cursor_down(1)?;

                        // Ensure that the cursor isn't beyond the end of the line.
                        if self.index > self.current_line_len() {
                            self.index = self.move_cursor_to_end()?;
                        }
                    }
                }
                Key::ArrowUp => {
                    if self.line > 1 {
                        self.line = self.move_cursor_up(1)?;

                        // Ensure that the cursor isn't beyond the end of the line.
                        if self.index > self.current_line_len() {
                            self.index = self.move_cursor_to_end()?;
                        }
                    }
                }
                Key::ArrowLeft => {
                    if self.index > 0 {
                        self.index = self.move_cursor_left(1)?;
                    } else if self.line > 1 {
                        // Move to the end of the previous line.
                        self.line = self.move_cursor_up(1)?;
                        self.index = self.move_cursor_to_end()?;
                    }
                }
                Key::ArrowRight => {
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
                    if self.index > 0 {
                        self.index = self.delete_char_before_cursor();
                        self.redraw()?;
                    } else if self.line > 1 {
                        // Backspace at the beginning of the line, so push the contents of
                        // the current line to the line above it, and remove the line.
                        self.draw_clear()?;

                        self.empty_padding += 1;
                        
                        // Push the content of the current line to the previous line.
                        let cbuf = self.buffers.remove(self.line - 1);
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
                    self.index = self.insert_char_at_cursor(c);
                    self.redraw()?;
                }
                Key::Escape => {
                    // Quick escape and finish the input.
                    self.move_cursor_to_bottom()?;
                    break
                }
                Key::Enter => {
                    if self.line == self.buffers.len() && self.current_line_len() == 0 {
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
                _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Unrecognized key"))?
            };
        }

        // Terminate the input.
        self.new_line()?;

        // If empty buffer, then return empty string.
        if self.buffers.is_empty() {
            return Ok(String::new())
        }

        // Otherwise, join the buffers together, putting a `\n` in between each line.
        let mut buf = self.buffers.remove(0).clone();
        for s in self.buffers {
            buf.push('\n');
            buf.push_str(&s);
        }
        Ok(buf)
    }

    fn delete_char_before_cursor(&mut self) -> usize {
        let idx = self.index;
        self.current_line_mut().remove(idx - 1);
        idx - 1
    }

    fn insert_char_at_cursor(&mut self, c: char) -> usize {
        if self.buffers.is_empty() {
            self.buffers.push(String::new());
        }

        let idx = self.index;
        let buf = self.current_line_mut();
        buf.insert(idx, c);
        idx + 1
    }

    fn draw(&self) -> io::Result<()> {
        if self.buffers.len() == 0 { return Ok(()) }

        for _ in 0..self.empty_padding {
            self.new_line()?;
        }
        
        for i in 0..self.buffers.len() - 1 {
            self.draw_prompt(i == 0)?;
            self.term.write_line(&self.buffers[i])?;
        }

        let last = &self.buffers[self.buffers.len() - 1];
        let last_len = last.len();
        self.draw_prompt(self.buffers.len() == 1)?;
        self.term.write_str(last)?;

        self.move_cursor_left(last_len)?;
        self.term.move_cursor_up(self.buffers.len() - self.line)?;
        self.term.write_str(&self.current_line()[0..self.index])?;
        Ok(())
    }

    fn draw_prompt(&self, draw_first: bool) -> io::Result<()> {
        if let Some((first, follow)) = &self.prompt {
            if draw_first {
                self.term.write_str(first)?;
            } else {
                self.term.write_str(follow)?;
            }
        }
        Ok(())
    }

    fn draw_clear(&self) -> io::Result<()> {
        self.move_cursor_to_bottom()?;
        self.term.clear_line()?;
        self.term.clear_last_lines(self.buffers.len() - 1 + self.empty_padding)?;
        Ok(())
    }

    fn redraw(&self) -> io::Result<()> {
        self.draw_clear()?;
        self.draw()
    }

    fn new_line(&self) -> io::Result<()> {
        self.term.write_line("")
    }

    fn move_cursor_to_bottom(&self) -> io::Result<usize> {
        self.term.move_cursor_down(self.buffers.len() - self.line + 1)?;
        Ok(self.buffers.len())
    }

    fn move_cursor_up(&self, n: usize) -> io::Result<usize> {
        self.term.move_cursor_up(n)?;
        Ok(self.line - 1)
    }

    fn move_cursor_down(&self, n: usize) -> io::Result<usize> {
        self.term.move_cursor_down(n)?;
        Ok(self.line + 1)
    }

    fn move_cursor_to_end(&self) -> io::Result<usize> {
        let len = self.current_line().len();
        if self.index > len {
            self.move_cursor_left(self.index - len)?;
        } else if self.index < len {
            self.move_cursor_right(len - self.index)?;
        }
        Ok(len)
    }

    fn move_cursor_to_start(&self) -> io::Result<usize> {
        self.move_cursor_left(self.index)?;
        Ok(0)
    }

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

    fn move_cursor_right(&self, n: usize) -> io::Result<usize> {
        self.term.write_str(&self.current_line()[self.index..self.index + n])?;
        Ok(self.index + 1)
    }
}