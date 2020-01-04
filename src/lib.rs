pub mod renderer;
pub mod keybindings;

use renderer::{RenderData, Renderer, full::FullRenderer};
use keybindings::Keybinding;

pub use crossterm;

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};

/// Multiline abstraction around a terminal.
pub struct Editor<'w> {
    pub buffers: Vec<String>,
    pub cursor: Cursor,
    renderer: Box<dyn 'w + Renderer>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    /// Current line of the cursor.
    pub line: usize,
    /// Current index of the cursor.
    pub index: usize,
}

impl<'w> Editor<'w> {
    /// Create a builder for `MultilineTerm`.
    #[inline]
    pub fn builder<'b>() -> MultilineTermBuilder<'b> {
        MultilineTermBuilder::default()
    }

    /// Return a mutable reference to the buffer of the terminal.
    #[inline]
    pub fn buffers_mut(&mut self) -> &mut Vec<String> {
        &mut self.buffers
    }

    /// Get a reference to current line of the cursor on the buffer.
    /// Unlike `current_line_mut`, this function will not allocate a new string
    /// if the buffer is empty, instead returning an empty string.
    pub fn current_line(&self) -> &str {
        if self.buffers.len() == 0 {
            return "";
        }
        &self.buffers[self.cursor.line]
    }

    /// Get a mutable reference to the current line of the cursor on the buffer.
    ///
    /// ### Warning
    /// This function will allocate a new `String` to the buffer if it is empty.
    pub fn current_line_mut(&mut self) -> &mut String {
        if self.buffers.is_empty() {
            let s = String::new();
            self.buffers.push(s);
            return &mut self.buffers[0];
        }
        &mut self.buffers[self.cursor.line]
    }

    #[doc(hidden)]
    #[inline]
    fn render_data<'b>(buffers: &'b Vec<String>, cursor: &'b Cursor) -> RenderData<'b> {
        RenderData { buffers, cursor }
    }

    /// Read multiple lines of input.
    ///
    /// ### Features
    /// * `Enter` on an empty last line will submit the input.
    /// * `Enter` on a non-empty line will create a new line.
    /// * `Backspace` at the beginning of the line to tappend the content
    ///   of the current line to the previous line.
    ///
    /// The returned result does not include the final empty line or trailing newline.
    pub fn read_multiline(mut self) -> Result<String> {
        self.renderer
            .draw(Self::render_data(&self.buffers, &self.cursor))?;
        self.renderer.flush()?;

        enable_raw_mode()?;
        loop {
            let take_more = keybindings::NormalKeybinding.read(&mut self)?;

            self.renderer.redraw(Self::render_data(&self.buffers, &self.cursor))?;
            self.renderer.flush()?;

            if take_more {
                continue;
            } else {
                break;
            }           
        }

        // Clear the last empty useless line.
        self.renderer.clear_line()?;
        self.renderer.flush()?;

        disable_raw_mode()?;

        // If empty buffer, then return empty string.
        if self.buffers.is_empty() {
            return Ok(String::new());
        }

        // Join the buffers together, putting a `\n` in between each line.
        // Ensure the capacity to avoid reallocations.
        let mut buf =
            String::with_capacity(self.buffers.iter().map(|x| x.len() + 1).sum::<usize>() - 1);
        buf.push_str(&self.buffers[0]);
        for s in &self.buffers[1..] {
            buf.push('\n');
            buf.push_str(&s);
        }

        Ok(buf)
    }

    #[doc(hidden)]
    fn delete_char_before_cursor(&mut self) -> usize {
        let idx = self.cursor.index;
        self.current_line_mut().remove(idx - 1);
        idx - 1
    }

    #[doc(hidden)]
    fn delete_char_after_cursor(&mut self) -> usize {
        let idx = self.cursor.index;
        self.current_line_mut().remove(idx);
        idx
    }
 
    #[doc(hidden)]
    fn insert_char_before_cursor(&mut self, c: char) -> usize {
        let idx = self.cursor.index;
        self.current_line_mut().insert(idx, c);
        idx + 1
    }

    // Returns an index that ensure that the cursor index is not overflowing the end.
    #[doc(hidden)]
    fn clamp_cursor_index(&self) -> usize {
        self.cursor.index.min(self.current_line().len())
    }
}

/// Builder struct for `MultilineTerm`.
#[derive(Default)]
pub struct MultilineTermBuilder<'w> {
    /// Initial buffer for the multiline terminal.
    buffers: Vec<String>,
    /// Initial line that the cursor is supposed to be set at.
    line: usize,
    /// Initial index that the cursor is supposed to be set at.
    index: usize,
    /// The renderer.
    renderer: Option<Box<dyn 'w + Renderer>>,
}

impl<'w> MultilineTermBuilder<'w> {
    /// Set the buffer that the terminal will be initialized with.
    #[inline]
    pub fn initial_buffers(mut self, buffers: Vec<String>) -> Self {
        self.buffers = buffers;
        self
    }

    /// Set what line the cursor will initially start at.
    #[inline]
    pub fn line(mut self, line: usize) -> Self {
        self.line = line;
        self
    }

    /// Set what index the cursor will initially start at.
    #[inline]
    pub fn index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn renderer(mut self, renderer: impl 'w + Renderer) -> Self {
        self.renderer = Some(Box::new(renderer));
        self
    }

    pub fn build(self) -> Editor<'w> {
        Editor {
            buffers: self.buffers,
            cursor: Cursor {
                line: self.line,
                index: self.index,
            },
            renderer: self
                .renderer
                .unwrap_or_else(|| Box::new(FullRenderer::default())),
        }
    }
}
