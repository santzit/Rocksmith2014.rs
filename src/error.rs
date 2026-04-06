use thiserror::Error;

/// All errors that can be returned by this library.
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid PSARC: {0}")]
    InvalidPsarc(String),

    #[error("Invalid SNG: {0}")]
    InvalidSng(String),

    #[error("XML parse error: {0}")]
    XmlParse(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Entry index {0} out of range (count: {1})")]
    IndexOutOfRange(usize, usize),
}

pub type Result<T> = std::result::Result<T, Error>;
