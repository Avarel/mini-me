pub mod data;
pub mod full;

use self::data::RenderData;

use crossterm::Result;

pub trait Renderer {
    fn draw(&mut self, data: RenderData) -> Result<()>;
    fn redraw(&mut self, data: RenderData) -> Result<()>;
    fn clear_draw(&mut self) -> Result<()>;
    fn clear_line(&mut self) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
}
