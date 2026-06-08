use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogError {
    #[error("Invalid log level: {0}")]
    InvalidLevel(String),

    #[error("Sink error: {0}")]
    SinkError(String),

    #[error("Invalid format string: {0}")]
    InvalidFormat(String),

    #[error("File rotation error: {0}")]
    RotationError(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, LogError>;
