use crate::MultilineTerm;
use crossterm::Result;

pub mod full;
pub mod lazy;

pub trait Renderer {
    fn draw(&self, term: &MultilineTerm) -> Result<()>;
    fn redraw(&self, term: &MultilineTerm) -> Result<()>;
    fn clear_draw(&self) -> Result<()>;
    fn clear_line(&self) -> Result<()>;
    fn flush(&self) -> Result<()>;
}
