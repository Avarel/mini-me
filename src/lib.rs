/// Module that contains core editor functionalities.
pub mod editor;
/// Module that handles rendering the editor.
pub mod renderer;

mod error;

mod util;

/// The `minime` result type.
pub use error::Result;

/// Re-export of crossterm crate.
pub use crossterm;