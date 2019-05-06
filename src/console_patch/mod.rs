mod common_term;

#[cfg(windows)]
mod windows_term;

#[cfg(unix)]
mod unix_term;

#[cfg(windows)]
pub use windows_term::*;

#[cfg(unix)]
pub use unix_term::*;