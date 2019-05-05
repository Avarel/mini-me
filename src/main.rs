use std::io::{self};

use console::{Term, Key};

fn main() -> io::Result<()> {
    let mut prompt = MultilineTerm::stdout();
    // prompt.buffers = vec![
    //     "hello there".to_owned(),
    //     "howtheheck are you doing".to_owned(),
    //     "memes".to_owned()
    // ];
    // prompt.cursor = Cursor {
    //     line: 2,
    //     index: 3,
    // };
    // prompt.prompt = Some(("  in > ".to_owned(), " ... > ".to_owned()));

    let result = prompt.read_prompt()?;
    println!("{}", result);
    Ok(())
}

struct MultilineTerm {
    term: Term,
    /// Buffers to draw
    pub buffers: Vec<String>,
    pub cursor: Cursor,
    pub prompt: Option<(String, String)>,
}

impl MultilineTerm {
    pub fn stdout() -> Self {
        Self {
            term: Term::stdout(),
            buffers: Vec::new(),
            cursor: Cursor { line: 1, index: 0 },
            prompt: None
        }
    }

    pub fn stderr() -> Self {
        Self {
            term: Term::stderr(),
            buffers: Vec::new(),
            cursor: Cursor { line: 1, index: 0 },
            prompt: None
        }
    }

    pub fn set_prompt(&mut self, prompt: Option<(String, String)>) {
        self.prompt = prompt
    }

    pub fn read_prompt(mut self) -> io::Result<String> {
        self.draw()?;
        loop {
            match self.term.read_key()? {
                Key::ArrowDown => {
                    if self.cursor.line < self.buffers.len() {
                        self.move_cursor_down(1)?;
                        self.cursor.line += 1;

                        let len = self.buffers[self.cursor.line - 1].len();
                        if self.cursor.index > len {
                            self.move_cursor_left(self.cursor.index - len)?;
                            self.cursor.index = len;
                        }
                    }
                }
                Key::ArrowUp => {
                    if self.cursor.line > 1 {
                        self.move_cursor_up(1)?;
                        self.cursor.line -= 1;

                        let len = self.buffers[self.cursor.line - 1].len();
                        if self.cursor.index > len {
                            self.move_cursor_left(self.cursor.index - len)?;
                            self.cursor.index = len;
                        }
                    }
                }
                Key::ArrowLeft => {
                    if self.cursor.index > 0 {
                        self.move_cursor_left(1)?;
                        self.cursor.index -= 1;
                    } else if self.cursor.line > 1 {
                        self.move_cursor_up(1)?;
                        self.cursor.line -= 1;
                        let len = self.buffers[self.cursor.line - 1].len();
                        self.move_cursor_right(len)?;
                        self.cursor.index = len;
                    }
                }
                Key::ArrowRight => {
                    let len = self.buffers[self.cursor.line - 1].len();
                    if self.cursor.index < len {
                        self.move_cursor_right(1)?;
                        self.cursor.index += 1;
                    } else if self.cursor.line < self.buffers.len() {
                        self.move_cursor_left(len)?;
                        self.move_cursor_down(1)?;
                        self.cursor.index = 0;
                        self.cursor.line += 1;
                    }
                }
                Key::Char(c) if c == '\x7f' => {
                    if self.cursor.index > 0 {
                        self.delete_char_before_cursor();
                        self.cursor.index -= 1;
                        self.redraw()?;
                    } else if self.cursor.line > 1 {
                        self.draw_clear()?;
                        self.term.write_line("")?;

                        let cbuf = self.buffers.remove(self.cursor.line - 1);
                        self.cursor.line -= 1;
                        
                        let nbuf = &mut self.buffers[self.cursor.line - 1];
                        self.cursor.index = nbuf.len();
                        nbuf.push_str(&cbuf);
                        
                        self.redraw()?;
                    }
                }
                Key::Char(c) => {
                    self.insert_char_at_cursor(c);
                    self.cursor.index += 1;
                    self.redraw()?;
                }
                Key::Escape => {
                    self.move_cursor_to_bottom()?;
                    break
                }
                Key::Enter => {
                    if self.cursor.line == self.buffers.len() && self.buffers[self.cursor.line - 1].len() == 0 {
                        self.buffers.remove(self.cursor.line - 1);
                        break
                    } else {
                        let cbuf = &mut self.buffers[self.cursor.line - 1];
                        let nbuf = cbuf.split_off(self.cursor.index);
                        self.buffers.insert(self.cursor.line, nbuf);
                        self.cursor.index = 0;
                        self.cursor.line += 1;
                        self.move_cursor_to_bottom()?;
                        self.term.write_line("")?;
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
        let buf = &mut self.buffers[self.cursor.line - 1];
        buf.remove(self.cursor.index - 1);
    }

    fn insert_char_at_cursor(&mut self, c: char) {
        if self.buffers.is_empty() {
            self.buffers.push(String::new());
        }

        let buf = &mut self.buffers[self.cursor.line - 1];
        buf.insert(self.cursor.index, c);
    }

    fn draw(&mut self) -> io::Result<()> {
        if self.buffers.len() == 0 { return Ok(()) }
        
        for i in 0..self.buffers.len() - 1 {
            if let Some((first, follow)) = &self.prompt {
                if i == 0 {
                    self.term.write_str(first)?;
                } else {
                    self.term.write_str(follow)?;
                }
            }
            self.term.write_line(&self.buffers[i])?;
        }

        let last = &self.buffers[self.buffers.len() - 1];
        let last_len = last.len();
        if let Some((_, follow)) = &self.prompt {
            self.term.write_str(follow)?;
        }
        self.term.write_str(last)?;

        self.move_cursor_left(last_len)?;
        self.term.move_cursor_up(self.buffers.len() - self.cursor.line)?;
        self.term.write_str(&self.buffers[self.cursor.line - 1][0..self.cursor.index])?;
        Ok(())
    }

    fn redraw(&mut self) -> io::Result<()> {
        self.draw_clear()?;
        self.draw()
    }

    fn move_cursor_to_bottom(&self) -> io::Result<()> {
        self.term.move_cursor_down(self.buffers.len() - self.cursor.line + 1)
    }

    fn move_cursor_up(&mut self, n: usize) -> io::Result<()> {
        self.term.move_cursor_up(n)
    }

    fn move_cursor_down(&mut self, n: usize) -> io::Result<()> {
        self.term.move_cursor_down(n)
    }

    fn move_cursor_left(&mut self, n: usize) -> io::Result<()> {
        for _ in 0..n {
            self.term.write_str("\x08")?;
        }
        Ok(())
    }

    fn move_cursor_right(&mut self, n: usize) -> io::Result<()> {
        self.term.write_str(&self.buffers[self.cursor.line - 1][self.cursor.index..self.cursor.index + n])?;
        Ok(())
    }

    fn draw_clear(&self) -> io::Result<()> {
        self.move_cursor_to_bottom()?;
        self.term.clear_line()?;
        self.term.clear_last_lines(self.buffers.len() - 1)?;
        Ok(())
    }
}

struct Cursor {
    /// Starts at 1
    pub line: usize,
    /// Starts at 0
    pub index: usize,
}
