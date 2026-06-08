use thiserror::Error;

#[derive(Error, Debug)]
pub enum TqdmError {
    #[error("Progress bar already finished")]
    AlreadyFinished,

    #[error("Invalid style template: {0}")]
    InvalidTemplate(String),

    #[error(transparent)]
    Indicatif(#[from] indicatif::style::TemplateError),
}

pub type Result<T> = std::result::Result<T, TqdmError>;
