use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("the data for key `{0}` is not available")]
    Terminal(#[from] crossterm::ErrorKind),
    #[allow(dead_code)]
    #[error("unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;
