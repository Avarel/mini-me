/// A module that contains keybindings for the editor.
pub mod keybindings;

use std::{
    borrow::Cow,
    io::{Read, Stdout},
};

use self::keybindings::Keybinding;
use crate::{
    renderer::{
        full::{CrosstermRenderer, DefaultRenderer},
        RenderData, Renderer,
    },
    util::{trimmed, Cursor},
    Result,
};

use ropey::Rope;

pub enum Selection {
    Focus(Cursor),
    Range {
        focus: Cursor,
        anchor: Cursor
    }
}

pub struct Editor<R> {
    cursor: Cursor,
    buf: Rope,
    renderer: R,
}

impl Default for Editor<DefaultRenderer<'static, Stdout>> {
    fn default() -> Self {
        Editor::with_renderer(CrosstermRenderer::default())
    }
}

impl<R: Renderer> Editor<R> {
    pub fn with_renderer(renderer: R) -> Self {
        Editor {
            buf: Rope::new(),
            cursor: Cursor::default(),
            renderer,
        }
    }

    pub fn set_contents(&mut self, reader: impl Read) -> Result<()> {
        self.buf = Rope::from_reader(reader)?;
        Ok(())
    }

    pub fn read(mut self, keybinding: impl Keybinding) -> Result<String> {
        loop {
            self.renderer
                .draw(RenderData::new(&self.buf, &self.cursor))?;
            self.renderer.flush()?;

            if !keybinding.read(&mut self)? {
                break;
            }
        }

        self.renderer.finish()?;

        Ok(trimmed(self.buf.slice(..)).to_string())
    }
}

impl<R> Editor<R> {
    pub fn line_count(&self) -> usize {
        self.buf.len_lines()
    }

    // pub fn insert_line(&mut self, line_idx: usize, string: &str) {
    //     let line_start = self.buf.line_to_char(line_idx);
    //     self.buf.insert(line_start, &string);
    // }

    // pub fn remove_line(&mut self, line_idx: usize) -> String {
    //     let line_start = self.buf.line_to_char(line_idx);
    //     let line_end = self.buf.line_to_char(line_idx + 1);
    //     let rm = self.buf.line_trimmed(line_idx).to_string();
    //     self.buf.remove(line_start..line_end);

    //     return rm;
    // }

    // pub fn push_line_str(&mut self, line_idx: usize, string: &str) {
    //     let line_end = self.buf.line_to_char(line_idx + 1) - 1;
    //     self.buf.insert(line_end, &string)
    // }

    pub fn ln_mut(&mut self) -> &mut usize {
        &mut self.cursor.ln
    }

    pub fn col_mut(&mut self) -> &mut usize {
        &mut self.cursor.col
    }

    pub fn ln(&self) -> usize {
        self.cursor.ln
    }

    pub fn col(&self) -> usize {
        self.cursor.col
    }

    pub fn clamp(&mut self) {
        self.cursor.col = self.cursor.col.min(self.curr_ln_len());
    }

    pub fn curr_ln_len(&self) -> usize {
        trimmed(self.buf.line(self.cursor.ln)).len_chars()
    }

    pub fn curr_ln(&self) -> Cow<str> {
        Cow::from(trimmed(self.buf.line(self.cursor.ln)))
    }

    pub fn curr_char(&self) -> char {
        self.buf.char(self.cursor_rope_idx(0))
    }

    pub fn move_up(&mut self) {
        if self.cursor.ln == 0 {
            self.cursor.col = 0;
        } else {
            self.cursor.ln -= 1;
        }
    }

    pub fn backspace(&mut self) {
        self.clamp();

        if self.cursor.col > 0 {
            self.delete_char(-1);
            self.cursor.col -= 1;
        } else if self.cursor.ln > 0 {
            let col = self.buf.line(self.cursor.ln - 1).len_chars();
            self.delete_char(-1);
            self.cursor.ln -= 1;
            self.cursor.col = col - 1;
        }
    }

    pub fn delete(&mut self) {
        self.clamp();

        if self.cursor.col < self.curr_ln_len() || self.cursor.ln + 1 < self.line_count() {
            self.delete_char(0);
        }
    }

    pub fn move_right(&mut self) {
        self.clamp();
        let len = self.curr_ln_len();
        if self.cursor.col < len {
            self.cursor.col += 1;
        } else if self.cursor.ln + 1 < self.line_count() {
            // Move to the beginning of the next line.
            self.cursor.ln += 1;
            self.cursor.col = 0;
        }
    }

    pub fn move_left(&mut self) {
        self.clamp();
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        } else if self.cursor.ln > 0 {
            // Move to the end of the previous line.
            self.cursor.ln -= 1;
            self.cursor.col = self.curr_ln_len();
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor.ln + 1 == self.line_count() {
            self.cursor.col = self.curr_ln_len();
        } else {
            self.cursor.ln += 1;
        }
    }

    pub fn move_to_col(&mut self, col: usize) {
        self.cursor.col = col;
    }

    pub fn move_to_top(&mut self) {
        self.cursor.ln = 0;
    }

    pub fn move_to_bottom(&mut self) {
        self.cursor.ln = self.line_count() - 1;
    }

    pub fn move_to_line_end(&mut self) {
        self.move_to_col(self.curr_ln_len());
    }

    pub fn delete_char(&mut self, offset: isize) {
        let z = self.cursor_rope_idx(offset);
        self.buf.remove(z..z + 1);
    }

    pub fn insert_char(&mut self, offset: isize, c: char) {
        let z = self.cursor_rope_idx(offset);
        self.buf.insert_char(z, c);
    }

    pub fn type_char(&mut self, c: char) {
        self.clamp();
        self.insert_char(0, c);
        if c == '\n' {
            self.cursor.col = 0;
            self.cursor.ln += 1;
        } else {
            self.cursor.col = self.cursor.col + 1;
        }
    }

    pub(crate) fn insert_str(&mut self, str: &str) {
        self.clamp();
        let z = self.cursor_rope_idx(0);
        self.buf.insert(z, str);

        let lines = str.lines().count().max(1);

        self.cursor.ln += lines - 1;
        self.cursor.col = self.cursor.col + str.lines().last().unwrap_or_default().len();
    }

    fn cursor_rope_idx(&self, offset: isize) -> usize {
        let idx = self.cursor.col;
        let line_start = self.buf.line_to_char(self.cursor.ln);
        let z = line_start + idx;
        // Take good care to not underflow subtract.
        // Also I'm keeping this and assuming that
        // all machines will use Two's Complement.
        z.wrapping_add(offset as usize)
    }
}
