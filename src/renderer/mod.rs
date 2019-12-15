use crate::Cursor;
use crossterm::Result;

pub mod full;
pub mod lazy;

pub trait Renderer {
    fn draw(&mut self, data: &RenderData) -> Result<()>;
    fn redraw(&mut self, data: &RenderData) -> Result<()>;
    fn clear_draw(&mut self) -> Result<()>;
    fn clear_line(&mut self) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
}

pub struct RenderData<'b> {
    pub buffers: &'b Vec<String>,
    pub cursor: &'b Cursor,
}

impl RenderData<'_> {
    pub fn current_line(&self) -> &str {
        if self.buffers.len() == 0 {
            return "";
        }
        &self.buffers[self.cursor.line]
    }

    pub fn line(&self, index: usize) -> &str {
        &self.buffers[index]
    }
}