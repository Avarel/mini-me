pub mod cursor;
pub mod keybindings;

use std::io::Stdout;

use crate::{
    ext::RopeExt,
    renderer::{data::RenderData, full::CrosstermRenderer, Renderer},
};

use crossterm::Result;
use ropey::Rope;

use self::{cursor::EditorCursor, keybindings::Keybinding};
use crate::Cursor;

/// Multiline abstraction around a terminal.
pub struct Editor<R> {
    cursor: Cursor,
    buf: Rope,
    renderer: R,
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
}

impl<R> Editor<R> {
    pub fn line_count(&self) -> usize {
        self.buf.len_lines()
    }

    pub fn cursor(&mut self) -> EditorCursor<'_, R> {
        EditorCursor { editor: self }
    }

    pub fn insert_line(&mut self, line_idx: usize, string: &str) {
        let line_start = self.buf.line_to_char(line_idx);
        self.buf.insert(line_start, &string);
    }

    // pub fn remove_line(&mut self, line_idx: usize) -> String {
    //     let line_start = self.buf.line_to_char(line_idx);
    //     let line_end = self.buf.line_to_char(line_idx + 1);
    //     let rm = self.buf.line_trimmed(line_idx).to_string();
    //     self.buf.remove(line_start..line_end);

    //     return rm;
    // }

    pub fn push_line_str(&mut self, line_idx: usize, string: &str) {
        let line_end = self.buf.line_to_char(line_idx + 1) - 1;
        self.buf.insert(line_end, &string)
    }
}
