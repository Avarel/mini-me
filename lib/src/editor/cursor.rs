use std::borrow::Cow;

use crate::util::trimmed;

use super::Editor;

pub struct EditorCursor<'e, R> {
    pub(super) editor: &'e mut Editor<R>,
}

impl<R> EditorCursor<'_, R> {
    pub fn ln_mut(&mut self) -> &mut usize {
        &mut self.editor.cursor.ln
    }

    pub fn col_mut(&mut self) -> &mut usize {
        &mut self.editor.cursor.col
    }

    pub fn ln(&self) -> usize {
        self.editor.cursor.ln
    }

    pub fn col(&self) -> usize {
        self.editor.cursor.col
    }

    pub fn clamp(&mut self) {
        self.editor.cursor.col = self.editor.cursor.col.min(self.current_line_len());
    }

    pub fn current_line_len(&self) -> usize {
        trimmed(self.editor.buf.line(self.editor.cursor.ln)).len_chars()
    }

    pub fn current_line(&self) -> Cow<str> {
        Cow::from(trimmed(self.editor.buf.line(self.editor.cursor.ln)))
    }

    pub fn curr_char(&self) -> char {
        self.editor.buf.char(self.cursor_rope_idx(0))
    }

    pub fn move_up(&mut self) {
        if self.editor.cursor.ln == 0 {
            self.editor.cursor.col = 0;
        } else {
            self.editor.cursor.ln -= 1;
        }
    }

    pub fn backspace(&mut self) {
        self.clamp();

        if self.editor.cursor.col > 0 {
            self.delete_char(-1);
            self.editor.cursor.col -= 1;
        } else if self.editor.cursor.ln > 0 {
            let col = self.editor.buf.line(self.editor.cursor.ln - 1).len_chars();
            self.delete_char(-1);
            self.editor.cursor.ln -= 1;
            self.editor.cursor.col = col - 1;
        }
    }

    pub fn delete(&mut self) {
        self.clamp();

        if self.editor.cursor.col < self.current_line_len()
            || self.editor.cursor.ln + 1 < self.editor.line_count()
        {
            self.delete_char(0);
        }
    }

    pub fn move_right(&mut self) {
        self.clamp();
        let len = self.current_line_len();
        if self.editor.cursor.col < len {
            self.editor.cursor.col += 1;
        } else if self.editor.cursor.ln + 1 < self.editor.line_count() {
            // Move to the beginning of the next line.
            self.editor.cursor.ln += 1;
            self.editor.cursor.col = 0;
        }
    }

    pub fn move_left(&mut self) {
        self.clamp();
        if self.editor.cursor.col > 0 {
            self.editor.cursor.col -= 1;
        } else if self.editor.cursor.ln > 0 {
            // Move to the end of the previous line.
            self.editor.cursor.ln -= 1;
            self.editor.cursor.col = self.current_line_len();
        }
    }

    pub fn move_down(&mut self) {
        if self.editor.cursor.ln + 1 == self.editor.line_count() {
            self.editor.cursor.col = self.current_line_len();
        } else {
            self.editor.cursor.ln += 1;
        }
    }

    pub fn move_to_col(&mut self, col: usize) {
        self.editor.cursor.col = col;
    }

    pub fn move_to_top(&mut self) {
        self.editor.cursor.ln = 0;
    }

    pub fn move_to_bottom(&mut self) {
        self.editor.cursor.ln = self.editor.line_count() - 1;
    }

    pub fn move_to_line_end(&mut self) {
        self.move_to_col(self.current_line_len());
    }

    pub fn delete_char(&mut self, offset: isize) {
        let z = self.cursor_rope_idx(offset);
        self.editor.buf.remove(z..z + 1);
    }

    pub fn insert_char(&mut self, offset: isize, c: char) {
        let z = self.cursor_rope_idx(offset);
        self.editor.buf.insert_char(z, c);
    }

    pub fn type_char(&mut self, c: char) {
        self.clamp();
        self.insert_char(0, c);
        if c == '\n' {
            self.editor.cursor.col = 0;
            self.editor.cursor.ln += 1;
        } else {
            self.editor.cursor.col = self.editor.cursor.col + 1;
        }
    }

    fn cursor_rope_idx(&self, offset: isize) -> usize {
        let idx = self.editor.cursor.col;
        let line_start = self.editor.buf.line_to_char(self.editor.cursor.ln);
        let z = line_start + idx;
        // Take good care to not underflow subtract.
        // Also I'm keeping this and assuming that
        // all machines will use Two's Complement.
        z.wrapping_add(offset as usize)
    }
}
