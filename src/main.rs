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

                        if self.index > self.current_line_len() {
                            self.index = self.move_cursor_to_end()?;
                        }
                    }
                }
                Key::ArrowUp => {
                    if self.line > 1 {
                        self.line = self.move_cursor_up(1)?;

                        if self.index > self.current_line_len() {
                            self.index = self.move_cursor_to_end()?;
                        }
                    }
                }
                Key::ArrowLeft => {
                    if self.index > 0 {
                        self.index = self.move_cursor_left(1)?;
                    } else if self.line > 1 {
                        self.line = self.move_cursor_up(1)?;
                        self.index = self.move_cursor_to_end()?;
                    }
                }
                Key::ArrowRight => {
                    let len = self.current_line().len();
                    if self.index < len {
                        self.index = self.move_cursor_right(1)?;
                    } else if self.line < self.buffers.len() {
                        self.line = self.move_cursor_down(1)?;
                        self.index = self.move_cursor_to_start()?;
                    }
                }
                Key::Char('\x7f') => {
                    if self.index > 0 {
                        self.delete_char_before_cursor();
                        self.index -= 1;
                        self.redraw()?;
                    } else if self.line > 1 {
                        self.draw_clear()?;
                        self.empty_padding += 1;
                        
                        let cbuf = self.buffers.remove(self.line - 1);
                        self.line -= 1;
                        
                        self.index = self.current_line_len();
                        self.current_line_mut().push_str(&cbuf);
                        
                        self.draw()?;
                    }
                }
                Key::Char(c) => {
                    self.insert_char_at_cursor(c);
                    self.index += 1;
                    self.redraw()?;
                }
                Key::Escape => {
                    self.move_cursor_to_bottom()?;
                    break
                }
                Key::Enter => {
                    if self.line == self.buffers.len() && self.current_line_len() == 0 {
                        self.buffers.remove(self.line - 1);
                        break
                    } else {
                        let cursor_idx = self.index;
                        let cbuf = self.current_line_mut();
                        let nbuf = cbuf.split_off(cursor_idx);
                        self.buffers.insert(self.line, nbuf);
                        self.index = 0;
                        self.line += 1;

                        self.move_cursor_to_bottom()?;
                        if self.empty_padding == 0 {
                            self.term.write_line("")?;
                        } else {
                            self.empty_padding -= 1;
                        }

                        self.redraw()?;
                    }
                }
                _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Unrecognized key"))?
            };
        }
        self.term.write_str("\n")?;

        if self.buffers.is_empty() {
            return Ok(String::new())
        }

        let mut buf = self.buffers.remove(0);
        for s in self.buffers {
            buf.push('\n');
            buf.push_str(&s);
        }
        Ok(buf)
    }

    fn delete_char_before_cursor(&mut self) {
        let idx = self.index;
        let buf = self.current_line_mut();
        buf.remove(idx - 1);
    }

    fn insert_char_at_cursor(&mut self, c: char) {
        if self.buffers.is_empty() {
            self.buffers.push(String::new());
        }

        let idx = self.index;
        let buf = self.current_line_mut();
        buf.insert(idx, c);
    }

    fn draw(&self) -> io::Result<()> {
        if self.buffers.len() == 0 { return Ok(()) }

        for _ in 0..self.empty_padding {
            self.term.write_line("")?;
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

    fn redraw(&self) -> io::Result<()> {
        self.draw_clear()?;
        self.draw()
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

    fn draw_clear(&self) -> io::Result<()> {
        self.move_cursor_to_bottom()?;
        self.term.clear_line()?;
        self.term.clear_last_lines(self.buffers.len() - 1 + self.empty_padding)?;
        Ok(())
    }
}