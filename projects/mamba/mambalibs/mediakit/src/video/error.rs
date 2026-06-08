//! Video processing error types.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VideoError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unsupported codec: {0}")]
    UnsupportedCodec(String),

    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("decode error: {0}")]
    DecodeError(String),

    #[error("encode error: {0}")]
    EncodeError(String),

    #[error("seek error: frame {0} out of range")]
    SeekOutOfRange(usize),

    #[error("no more frames")]
    EndOfStream,
}

pub type Result<T> = std::result::Result<T, VideoError>;
