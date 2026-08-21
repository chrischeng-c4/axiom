use crate::source::span::Span;

pub type Result<T> = std::result::Result<T, MambaError>;

#[derive(Debug, thiserror::Error)]
pub enum MambaError {
    #[error("syntax error at {span}: {message}")]
    Syntax { span: Span, message: String },

    #[error("type error at {span}: {message}")]
    Type { span: Span, message: String },

    #[error("name error at {span}: {message}")]
    Name { span: Span, message: String },

    #[error("codegen error: {message}")]
    Codegen { message: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl Clone for MambaError {
    fn clone(&self) -> Self {
        match self {
            Self::Syntax { span, message } => Self::Syntax {
                span: *span,
                message: message.clone(),
            },
            Self::Type { span, message } => Self::Type {
                span: *span,
                message: message.clone(),
            },
            Self::Name { span, message } => Self::Name {
                span: *span,
                message: message.clone(),
            },
            Self::Codegen { message } => Self::Codegen {
                message: message.clone(),
            },
            Self::Io(error) => Self::Io(std::io::Error::new(error.kind(), error.to_string())),
            Self::Other(message) => Self::Other(message.clone()),
        }
    }
}

impl MambaError {
    pub fn syntax(span: Span, message: impl Into<String>) -> Self {
        Self::Syntax {
            span,
            message: message.into(),
        }
    }

    pub fn type_err(span: Span, message: impl Into<String>) -> Self {
        Self::Type {
            span,
            message: message.into(),
        }
    }

    pub fn name(span: Span, message: impl Into<String>) -> Self {
        Self::Name {
            span,
            message: message.into(),
        }
    }

    pub fn codegen(message: impl Into<String>) -> Self {
        Self::Codegen {
            message: message.into(),
        }
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            Self::Syntax { span, .. } => Some(*span),
            Self::Type { span, .. } => Some(*span),
            Self::Name { span, .. } => Some(*span),
            _ => None,
        }
    }
}
