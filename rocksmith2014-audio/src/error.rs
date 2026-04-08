use thiserror::Error;

/// Errors that can occur during audio operations.
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ww2ogg process failed with output:\n{0}")]
    Ww2OggFailed(String),

    #[error("revorb process failed with {0}")]
    RevorbFailed(String),

    #[error("WWise conversion failed with exit code {code}. {output}")]
    WwiseFailed { code: i32, output: String },

    #[error("Could not find Wwise Console executable")]
    WwiseNotFound,

    #[error("The file: \"{0}\" does not exist")]
    FileNotFound(String),

    #[error("Unsupported Wwise version ({0}).\nMust be major version 2019/2021/2022/2023")]
    UnsupportedWwiseVersion(String),

    #[error("Wwise conversion is not supported on this OS")]
    UnsupportedOs,

    #[error("Could not find converted Wwise audio file")]
    WwiseFileNotFound,

    #[error("Could not detect audio file type from extension")]
    UnknownAudioExtension,

    #[error(
        "Path to Wwise console executable appears to be wrong.\n\
         It should be to WwiseConsole.exe on Windows or WwiseConsole.sh on macOS"
    )]
    InvalidCliPath,

    #[error("Only vorbis, wave and FLAC files are supported")]
    UnsupportedAudioFormat,

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AudioError>;
