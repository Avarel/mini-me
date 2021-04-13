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

pub struct Editor<R> {
    /// The focus cursor of the selection.
    pub focus: Cursor,
    /// The anchor cursor of the selection.
    pub anchor: Option<Cursor>,
    buf: Rope,
    renderer: R,
}

impl Default for Editor<DefaultRenderer<'static, Stdout>> {
    fn default() -> Self {
        Editor::with_renderer(CrosstermRenderer::default())
    }
}

impl<R: Renderer> Editor<R> {
    /// Create an editor with a specific renderer.
    pub fn with_renderer(renderer: R) -> Self {
        Editor {
            buf: Rope::new(),
            focus: Cursor::default(),
            anchor: None,
            renderer,
        }
    }

    /// Set the content of the editor.
    pub fn set_contents(&mut self, reader: impl Read) -> Result<()> {
        self.buf = Rope::from_reader(reader)?;
        Ok(())
    }

    /// Activate the editor and renderer, and read the input.
    pub fn read(mut self, keybinding: impl Keybinding) -> Result<String> {
        loop {
            self.renderer
                .draw(RenderData::new(&self.buf, self.focus, self.anchor))?;
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
    /// Get the number of lines.
    pub fn line_count(&self) -> usize {
        self.buf.len_lines()
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

        return rm;
    }

    // pub fn push_line_str(&mut self, line_idx: usize, string: &str) {
    //     let line_end = self.buf.line_to_char(line_idx + 1) - 1;
    //     self.buf.insert(line_end, &string)
    // }

    /// Unanchor if the focus is where the anchor is.
    pub fn fix_anchor(&mut self) {
        if self.anchor == Some(self.focus) {
            self.anchor = None;
        }
    }

    /// Anchor if there was not already an anchor, or unanchor.
    pub fn set_anchor(&mut self, anchored: bool) {
        if anchored {
            if self.anchor == None {
                self.anchor = Some(self.focus);
            }
        } else {
            self.anchor = None
        }
    }

    /// Clamp the cursor into valid indexing range on the current line.
    pub fn clamp(&mut self) {
        self.focus.col = self.focus.col.min(self.curr_ln_len());
    }

    /// Get the length of the current line.
    pub fn curr_ln_len(&self) -> usize {
        trimmed(self.buf.line(self.focus.ln)).len_chars()
    }

    /// Get a character iterator of the current line.
    pub fn curr_ln_chars(&self) -> impl Iterator<Item=char> + '_ {
        trimmed(self.buf.line(self.focus.ln)).chars()
    }

    /// Get the current line.
    pub fn curr_ln(&self) -> Cow<str> {
        Cow::from(trimmed(self.buf.line(self.focus.ln)))
    }

    /// Get the current selection of text.
    pub fn curr_sel(&self) -> Option<Cow<str>> {
        if let Some(anchor) = self.anchor {
            let anchor_idx = self.rope_idx(anchor, 0);
            let focus_idx = self.rope_idx(self.focus, 0);
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
        self.buf.char(self.rope_idx(self.focus, 0))
    }

    fn delete_ln_range(&mut self, start: usize, end: usize) {
        let idx = self.buf.line_to_char(self.focus.ln);
        self.buf.remove((idx + start)..(idx + end));
        if self.focus.col >= end {
            self.focus.col -= end - start;
        }
    }

    fn delete_selection(&mut self, focus: Cursor, anchor: Cursor) {
        let anchor_idx = self.rope_idx(anchor, 0);
        let focus_idx = self.rope_idx(focus, 0);
        if focus_idx < anchor_idx {
            self.buf.remove(focus_idx..anchor_idx)
        } else {
            self.focus = self.anchor.unwrap();
            self.buf.remove(anchor_idx..focus_idx)
        }
        self.anchor = None;
    }

    /// Execute a backspace.
    pub fn backspace(&mut self) {
        self.clamp();

        if let Some(anchor) = self.anchor {
            self.delete_selection(self.focus, anchor);
        } else if self.focus.col > 0 {
            self.delete_char(-1);
            self.focus.col -= 1;
        } else if self.focus.ln > 0 {
            let col = self.buf.line(self.focus.ln - 1).len_chars();
            self.delete_char(-1);
            self.focus.ln -= 1;
            self.focus.col = col - 1;
        }
    }

    /// Execute a delete.
    pub fn delete(&mut self) {
        self.clamp();

        if let Some(anchor) = self.anchor {
            self.delete_selection(self.focus, anchor);
        } else if self.focus.col < self.curr_ln_len() || self.focus.ln + 1 < self.line_count() {
            self.delete_char(0);
        }
    }

    /// Move the cursor right.
    pub fn move_right(&mut self, anchored: bool) {
        self.clamp();
        self.set_anchor(anchored);
        let len = self.curr_ln_len();
        if self.focus.col < len {
            self.focus.col += 1;
        } else if self.focus.ln + 1 < self.line_count() {
            // Move to the beginning of the next line.
            self.focus.ln += 1;
            self.focus.col = 0;
        }
        self.fix_anchor();
    }

    /// Move the cursor left.
    pub fn move_left(&mut self, anchored: bool) {
        self.clamp();
        self.set_anchor(anchored);
        if self.focus.col > 0 {
            self.focus.col -= 1;
        } else if self.focus.ln > 0 {
            // Move to the end of the previous line.
            self.focus.ln -= 1;
            self.focus.col = self.curr_ln_len();
        }
        self.fix_anchor();
    }

    /// Move the cursor up.
    pub fn move_up(&mut self, anchored: bool) {
        self.set_anchor(anchored);
        if self.focus.ln == 0 {
            self.focus.col = 0;
        } else {
            self.focus.ln -= 1;
        }
        self.fix_anchor();
    }

    /// Move the cursor down.
    pub fn move_down(&mut self, anchored: bool) {
        self.set_anchor(anchored);
        if self.focus.ln + 1 == self.line_count() {
            self.focus.col = self.curr_ln_len();
        } else {
            self.focus.ln += 1;
        }
        self.fix_anchor();
    }

    /// Move the cursor to a column.
    pub fn move_to_col(&mut self, col: usize, anchored: bool) {
        self.set_anchor(anchored);
        self.focus.col = col;
        self.fix_anchor();
    }

    /// Move the cursor to the top of the buffer.
    pub fn move_to_top(&mut self) {
        self.focus.ln = 0;
    }

    /// Move the cursor to the bottom of the buffer.
    pub fn move_to_bottom(&mut self) {
        self.focus.ln = self.line_count() - 1;
    }

    /// Move the cursor to the end of the current line.
    pub fn move_to_line_end(&mut self, anchored: bool) {
        self.move_to_col(self.curr_ln_len(), anchored);
    }

    /// Delete a character offset from the cursor.
    pub fn delete_char(&mut self, offset: isize) {
        let z = self.rope_idx(self.focus, offset);
        self.buf.remove(z..=z);
    }

    pub fn insert_char(&mut self, offset: isize, c: char) {
        let z = self.rope_idx(self.focus, offset);
        self.buf.insert_char(z, c);
    }

    /// Type a character at the cursor.
    pub fn type_char(&mut self, c: char) {
        self.clamp();
        if let Some(anchor) = self.anchor {
            self.delete_selection(self.focus, anchor);
        }
        self.insert_char(0, c);
        if c == '\n' {
            self.focus.col = 0;
            self.focus.ln += 1;
        } else {
            self.focus.col = self.focus.col + 1;
        }
    }

    /// Insert a string at the cursor.
    pub(crate) fn insert_str(&mut self, str: &str) {
        self.clamp();
        if let Some(anchor) = self.anchor {
            self.delete_selection(self.focus, anchor);
        }
        let z = self.rope_idx(self.focus, 0);
        self.buf.insert(z, str);

        let lines = str.lines().count().max(1);

        self.focus.ln += lines - 1;
        self.focus.col = self.focus.col + str.lines().last().unwrap_or_default().len();
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
