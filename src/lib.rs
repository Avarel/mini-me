pub mod editor;
pub mod renderer;
pub use crossterm;

mod ext;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    /// Current line of the cursor.
    pub ln: usize,
    /// Current index of the cursor.
    pub col: usize,
}