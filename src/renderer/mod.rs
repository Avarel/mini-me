use crate::Cursor;
use crossterm::Result;

pub mod full;
pub mod lazy;

pub trait Renderer {
    fn draw(&mut self, term: &RenderData) -> Result<()>;
    fn redraw(&mut self, term: &RenderData) -> Result<()>;
    fn clear_draw(&mut self) -> Result<()>;
    fn clear_line(&mut self) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
}

pub struct RenderData<'b> {
    pub buffers: &'b Vec<String>,
    pub cursor: &'b Cursor,
}

impl RenderData<'_> {
    #[inline]
    fn current_line_len(&self) -> usize {
        self.current_line().len()
    }

    pub fn current_line(&self) -> &str {
        if self.buffers.len() == 0 {
            return "";
        }
        &self.buffers[self.cursor.line]
    }
}