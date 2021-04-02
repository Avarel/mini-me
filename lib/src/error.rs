use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Terminal error: {0}")]
    Terminal(#[from] crossterm::ErrorKind),
    #[allow(dead_code)]
    #[error("Unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;
