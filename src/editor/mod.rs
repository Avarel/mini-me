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
    focus: Cursor,
    anchor: Option<Cursor>,
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
            focus: Cursor::default(),
            anchor: None,
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

    // Unanchor if the focus is where the anchor is.
    fn fix_anchor(&mut self) {
        if self.anchor == Some(self.focus) {
            self.anchor = None;
        }
    }

    // Anchor if there was not already an anchor, or unanchor.
    fn anchor(&mut self, anchored: bool) {
        if anchored {
            if self.anchor == None {
                self.anchor = Some(self.focus);
            }
        } else {
            self.anchor = None
        }
    }

    pub fn ln_mut(&mut self) -> &mut usize {
        &mut self.focus.ln
    }

    pub fn col_mut(&mut self) -> &mut usize {
        &mut self.focus.col
    }

    pub fn ln(&self) -> usize {
        self.focus.ln
    }

    pub fn col(&self) -> usize {
        self.focus.col
    }

    pub fn clamp(&mut self) {
        self.focus.col = self.focus.col.min(self.curr_ln_len());
    }

    pub fn curr_ln_len(&self) -> usize {
        trimmed(self.buf.line(self.focus.ln)).len_chars()
    }

    pub fn curr_ln(&self) -> Cow<str> {
        Cow::from(trimmed(self.buf.line(self.focus.ln)))
    }

    pub fn curr_char(&self) -> char {
        self.buf.char(self.rope_idx(self.focus, 0))
    }

    fn delete_range(&mut self, focus: Cursor, anchor: Cursor) {
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

    pub fn backspace(&mut self) {
        self.clamp();

        if let Some(anchor) = self.anchor {
            self.delete_range(self.focus, anchor);
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

    pub fn delete(&mut self) {
        self.clamp();

        if let Some(anchor) = self.anchor {
            self.delete_range(self.focus, anchor);
        } else if self.focus.col < self.curr_ln_len() || self.focus.ln + 1 < self.line_count() {
            self.delete_char(0);
        }
    }

    pub fn move_right(&mut self, anchored: bool) {
        self.clamp();
        self.anchor(anchored);
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

    pub fn move_left(&mut self, anchored: bool) {
        self.clamp();
        self.anchor(anchored);
        if self.focus.col > 0 {
            self.focus.col -= 1;
        } else if self.focus.ln > 0 {
            // Move to the end of the previous line.
            self.focus.ln -= 1;
            self.focus.col = self.curr_ln_len();
        }
        self.fix_anchor();
    }

    pub fn move_up(&mut self, anchored: bool) {
        self.anchor(anchored);
        if self.focus.ln == 0 {
            self.focus.col = 0;
        } else {
            self.focus.ln -= 1;
        }
        self.fix_anchor();
    }

    pub fn move_down(&mut self, anchored: bool) {
        self.anchor(anchored);
        if self.focus.ln + 1 == self.line_count() {
            self.focus.col = self.curr_ln_len();
        } else {
            self.focus.ln += 1;
        }
        self.fix_anchor();
    }

    pub fn move_to_col(&mut self, col: usize, anchored: bool) {
        self.anchor(anchored);
        self.focus.col = col;
        self.fix_anchor();
    }

    pub fn move_to_top(&mut self) {
        self.focus.ln = 0;
    }

    pub fn move_to_bottom(&mut self) {
        self.focus.ln = self.line_count() - 1;
    }

    pub fn move_to_line_end(&mut self, anchored: bool) {
        self.move_to_col(self.curr_ln_len(), anchored);
    }

    pub fn delete_char(&mut self, offset: isize) {
        let z = self.rope_idx(self.focus, offset);
        self.buf.remove(z..=z);
    }

    pub fn insert_char(&mut self, offset: isize, c: char) {
        let z = self.rope_idx(self.focus, offset);
        self.buf.insert_char(z, c);
    }

    pub fn type_char(&mut self, c: char) {
        self.clamp();
        if let Some(anchor) = self.anchor {
            self.delete_range(self.focus, anchor);
        }
        self.insert_char(0, c);
        if c == '\n' {
            self.focus.col = 0;
            self.focus.ln += 1;
        } else {
            self.focus.col = self.focus.col + 1;
        }
    }

    pub(crate) fn insert_str(&mut self, str: &str) {
        self.clamp();
        if let Some(anchor) = self.anchor {
            self.delete_range(self.focus, anchor);
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
