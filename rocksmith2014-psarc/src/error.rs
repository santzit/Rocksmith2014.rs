use thiserror::Error;

/// Errors that can occur when working with PSARC archives.
#[derive(Debug, Error)]
pub enum PsarcError {
    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The PSARC header is invalid.
    #[error("Invalid PSARC header: {0}")]
    InvalidHeader(String),

    /// The requested file was not found in the archive.
    #[error("File not found in PSARC: '{0}'")]
    FileNotFound(String),

    /// The TOC decryption failed.
    #[error("TOC decryption failed: incorrect TOC size")]
    DecryptionFailed,

    /// An unsupported feature was encountered.
    #[error("Unsupported feature: {0}")]
    Unsupported(String),
}

/// Convenience type alias for `Result<T, PsarcError>`.
pub type Result<T> = std::result::Result<T, PsarcError>;
