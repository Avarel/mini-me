/// A module that contains keybindings for the editor.
pub mod keybindings;
pub mod selection;

use std::{borrow::Cow, io::Read};

use self::{keybindings::Keybinding, selection::{Cursor, Selection}};
use crate::{Result, renderer::{Renderer}, util::trimmed};

use ropey::Rope;

pub struct Editor {
    pub selection: Selection,
    pub(crate) buf: Rope,
    pub altscreen: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            buf: Rope::new(),
            selection: Selection::default(),
            altscreen: false,
        }
    }
}

impl Editor {
    /// Set the content of the editor.
    pub fn set_contents(&mut self, reader: impl Read) -> Result<()> {
        self.buf = Rope::from_reader(reader)?;
        Ok(())
    }

    /// Get the content of the editor.
    pub fn contents(&self) -> String {
        trimmed(self.buf.slice(..)).to_string()
    }

    /// Activate the editor and renderer, and read the input.
    pub fn read(&mut self, keybinding: impl Keybinding, mut renderer: impl Renderer) -> Result<()> {
        loop {
            renderer.draw(self)?;
            renderer.flush()?;

            if !keybinding.read(self)? {
                break;
            }
        }

        renderer.finish()?;

        Ok(())
    }

    /// Get the number of lines.
    pub fn line_count(&self) -> usize {
        self.buf.len_lines()
    }

    /// Get the number of characters.
    pub fn char_count(&self) -> usize {
        self.buf.len_chars()
    }

    pub fn line(&self, index: usize) -> Cow<str> {
        trimmed(self.buf.line(index)).into()
    }

    // pub fn insert_line(&mut self, line_idx: usize, string: &str) {
    //     let line_start = self.buf.line_to_char(line_idx);
    //     self.buf.insert(line_start, &string);
    // }

    #[cfg(feature = "unstable")]
    pub fn remove_line(&mut self, line_idx: usize) -> String {
        let line_start = self.buf.line_to_char(line_idx);
        let line_end = self.buf.line_to_char(line_idx + 1);
        let rm = self.buf.line(line_idx).to_string();
        self.buf.remove(line_start..line_end);

        if self.selection.focus.ln == line_idx {
            self.selection.focus.col = 0;
        }

        return rm;
    }

    // pub fn push_line_str(&mut self, line_idx: usize, string: &str) {
    //     let line_end = self.buf.line_to_char(line_idx + 1) - 1;
    //     self.buf.insert(line_end, &string)
    // }

    /// Clamp the cursor into valid indexing range on the current line.
    pub fn clamp(&mut self) {
        self.selection.focus.col = self.selection.focus.col.min(self.curr_ln_len());
    }

    /// Get the length of the current line.
    pub fn curr_ln_len(&self) -> usize {
        trimmed(self.buf.line(self.selection.focus.ln)).len_chars()
    }

    /// Get a character iterator of the current line.
    pub fn curr_ln_chars(&self) -> impl Iterator<Item = char> + '_ {
        trimmed(self.buf.line(self.selection.focus.ln)).chars()
    }

    pub fn last_line(&self) -> Cow<str> {
        self.line(self.buf.len_lines() - 1)
    }

    /// Get the current line.
    pub fn curr_ln(&self) -> Cow<str> {
        Cow::from(trimmed(self.buf.line(self.selection.focus.ln)))
    }

    /// Get the current selection of text.
    pub fn curr_sel(&self) -> Option<Cow<str>> {
        if let Some(anchor) = self.selection.anchor {
            let anchor_idx = self.rope_idx(anchor, 0);
            let focus_idx = self.rope_idx(self.selection.focus, 0);
            let slice = if focus_idx < anchor_idx {
                self.buf.slice(focus_idx..anchor_idx)
            } else {
                self.buf.slice(anchor_idx..focus_idx)
            };
            Some(Cow::from(trimmed(slice)))
        } else {
            None
        }
    }

    /// Get the character the focus is pointed at.
    pub fn curr_char(&self) -> char {
        self.buf.char(self.rope_idx(self.selection.focus, 0))
    }

    fn delete_ln_range(&mut self, start: usize, end: usize) {
        let idx = self.buf.line_to_char(self.selection.focus.ln);
        self.buf.remove((idx + start)..(idx + end));
        if self.selection.focus.col >= end {
            self.selection.focus.col -= end - start;
        }
    }

    fn delete_selection(&mut self, focus: Cursor, anchor: Cursor) {
        let anchor_idx = self.rope_idx(anchor, 0);
        let focus_idx = self.rope_idx(focus, 0);
        if focus_idx < anchor_idx {
            self.buf.remove(focus_idx..anchor_idx)
        } else {
            self.selection.focus = self.selection.anchor.unwrap();
            self.buf.remove(anchor_idx..focus_idx)
        }
        self.selection.anchor = None;
    }

    /// Execute a backspace.
    pub fn backspace(&mut self) {
        self.clamp();

        if let Some(anchor) = self.selection.anchor {
            self.delete_selection(self.selection.focus, anchor);
        } else if self.selection.focus.col > 0 {
            self.delete_char(-1);
            self.selection.focus.col -= 1;
        } else if self.selection.focus.ln > 0 {
            let col = self.buf.line(self.selection.focus.ln - 1).len_chars();
            self.delete_char(-1);
            self.selection.focus.ln -= 1;
            self.selection.focus.col = col - 1;
        }
    }

    /// Execute a delete.
    pub fn delete(&mut self) {
        self.clamp();

        if let Some(anchor) = self.selection.anchor {
            self.delete_selection(self.selection.focus, anchor);
        } else if self.selection.focus.col < self.curr_ln_len() || self.selection.focus.ln + 1 < self.line_count() {
            self.delete_char(0);
        }
    }

    /// Move the cursor right.
    pub fn move_right(&mut self, anchored: bool) {
        self.clamp();
        self.selection.set_anchor(anchored);
        let len = self.curr_ln_len();
        if self.selection.focus.col < len {
            self.selection.focus.col += 1;
        } else if self.selection.focus.ln + 1 < self.line_count() {
            // Move to the beginning of the next line.
            self.selection.focus.ln += 1;
            self.selection.focus.col = 0;
        }
        self.selection.fix_anchor();
    }

    /// Move the cursor left.
    pub fn move_left(&mut self, anchored: bool) {
        self.clamp();
        self.selection.set_anchor(anchored);
        if self.selection.focus.col > 0 {
            self.selection.focus.col -= 1;
        } else if self.selection.focus.ln > 0 {
            // Move to the end of the previous line.
            self.selection.focus.ln -= 1;
            self.selection.focus.col = self.curr_ln_len();
        }
        self.selection.fix_anchor();
    }

    /// Move the cursor up.
    pub fn move_up(&mut self, anchored: bool) {
        self.selection.set_anchor(anchored);
        if self.selection.focus.ln == 0 {
            self.selection.focus.col = 0;
        } else {
            self.selection.focus.ln -= 1;
        }
        self.selection.fix_anchor();
    }

    /// Move the cursor down.
    pub fn move_down(&mut self, anchored: bool) {
        self.selection.set_anchor(anchored);
        if self.selection.focus.ln + 1 == self.line_count() {
            self.selection.focus.col = self.curr_ln_len();
        } else {
            self.selection.focus.ln += 1;
        }
        self.selection.fix_anchor();
    }

    /// Move the cursor to a column.
    pub fn move_to_col(&mut self, col: usize, anchored: bool) {
        self.selection.set_anchor(anchored);
        self.selection.focus.col = col;
        self.selection.fix_anchor();
    }

    /// Move the cursor to the top of the buffer.
    pub fn move_to_top(&mut self) {
        self.selection.focus.ln = 0;
    }

    /// Move the cursor to the bottom of the buffer.
    pub fn move_to_bottom(&mut self) {
        self.selection.focus.ln = self.line_count() - 1;
    }

    /// Move the cursor to the end of the current line.
    pub fn move_to_line_end(&mut self, anchored: bool) {
        self.move_to_col(self.curr_ln_len(), anchored);
    }

    /// Delete a character offset from the cursor.
    pub fn delete_char(&mut self, offset: isize) {
        let z = self.rope_idx(self.selection.focus, offset);
        self.buf.remove(z..=z);
    }

    pub fn insert_char(&mut self, offset: isize, c: char) {
        let z = self.rope_idx(self.selection.focus, offset);
        self.buf.insert_char(z, c);
    }

    /// Type a character at the cursor.
    pub fn type_char(&mut self, c: char) {
        self.clamp();
        if let Some(anchor) = self.selection.anchor {
            self.delete_selection(self.selection.focus, anchor);
        }
        self.insert_char(0, c);
        if c == '\n' {
            self.selection.focus.col = 0;
            self.selection.focus.ln += 1;
        } else {
            self.selection.focus.col = self.selection.focus.col + 1;
        }
    }

    /// Insert a string at the cursor.
    pub(crate) fn insert_str(&mut self, str: &str) {
        self.clamp();
        if let Some(anchor) = self.selection.anchor {
            self.delete_selection(self.selection.focus, anchor);
        }
        let z = self.rope_idx(self.selection.focus, 0);
        self.buf.insert(z, str);

        let lines = str.lines().count().max(1);

        self.selection.focus.ln += lines - 1;
        self.selection.focus.col = self.selection.focus.col + str.lines().last().unwrap_or_default().len();
    }

    fn rope_idx(&self, cursor: Cursor, offset: isize) -> usize {
        let idx = cursor.col;
        let line_start = self.buf.line_to_char(cursor.ln);
        let z = line_start + idx;
        // Take good care to not underflow subtract.
        // Also I'm keeping this and assuming that
        // all machines will use Two's Complement.
        z.wrapping_add(offset as usize)
    }
}
