use std::io::Stdout;

use crate::{
    ext::RopeExt,
    keybindings::Keybinding,
    renderer::{data::RenderData, full::CrosstermRenderer, Renderer},
};

use crossterm::Result;
use ropey::Rope;

/// Multiline abstraction around a terminal.
pub struct Editor<R> {
    pub cursor: Cursor,
    buf: Rope,
    renderer: R,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    /// Current line of the cursor.
    pub ln: usize,
    /// Current index of the cursor.
    pub col: usize,
}

impl Editor<CrosstermRenderer<'_, Stdout>> {
    pub fn new() -> Self {
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

    pub fn line_count(&self) -> usize {
        self.buf.len_lines()
    }

    pub fn current_line_len(&self) -> usize {
        self.buf.line_trimmed(self.cursor.ln).len_chars()
    }

    pub fn read(mut self, keybinding: impl Keybinding) -> Result<String> {
        loop {
            self.renderer
                .redraw(RenderData::new(&self.buf, &self.cursor))?;
            self.renderer.flush()?;

            if !keybinding.read(&mut self)? {
                break;
            }
        }

        self.renderer.finish()?;

        Ok(self.buf.trimmed().to_string())
    }

    fn cursor_rope_idx(&self, offset: isize) -> usize {
        let idx = self.cursor.col;
        let line_start = self.buf.line_to_char(self.cursor.ln);
        let z = line_start + idx;
        z + offset as usize
    }

    pub fn delete_char_at_cursor(&mut self, offset: isize) -> usize {
        let z = self.cursor_rope_idx(offset);
        self.buf.remove(z..z + 1);
        self.cursor.col + offset as usize
    }

    pub fn insert_char_at_cursor(&mut self, offset: isize, c: char) -> usize {
        let z = self.cursor_rope_idx(offset);
        self.buf.insert_char(z, c);
        self.cursor.col + offset as usize + 1
    }

    pub fn insert_line(&mut self, line_idx: usize, string: &str) {
        self.buf.insert_line(line_idx, string)
    }

    pub fn remove_line(&mut self, line_idx: usize) -> String {
        self.buf.remove_line(line_idx)
    }

    pub fn push_line_str(&mut self, line_idx: usize, string: &str) {
        let line_end = self.buf.line_to_char(line_idx + 1) - 1;
        self.buf.insert(line_end, &string)
    }

    pub fn split_line(&mut self, line_idx: usize, cursor_idx: usize) {
        let line_start = self.buf.line_to_char(line_idx);
        let z = line_start + cursor_idx;
        self.buf.insert_char(z, '\n');
    }
}
