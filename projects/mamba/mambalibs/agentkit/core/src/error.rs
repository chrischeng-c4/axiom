use thiserror::Error;

/// Unified error type for cclab-nova
#[derive(Error, Debug)]
pub enum NovaError {
    // Agent errors
    #[error("LLM provider error: {0}")]
    LLMError(String),

    #[error("Tool execution failed: {0}")]
    ToolError(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool requires approval: {0}")]
    ApprovalRequired(String),

    #[error("Approval denied by user: {0}")]
    ApprovalDenied(String),

    #[error("Security policy violation: {0}")]
    SecurityViolation(String),

    #[error("Path not allowed: {0}")]
    PathNotAllowed(String),

    #[error("Command not allowed: {0}")]
    CommandNotAllowed(String),

    #[error("Maximum turns reached: {0}")]
    MaxTurnsReached(u32),

    #[error("Maximum revisions exceeded: {0}")]
    MaxRevisionsExceeded(u32),

    #[error("Malformed LLM response: {0}")]
    MalformedLLMResponse(String),

    #[error("Platform integration error: {0}")]
    PlatformError(String),

    #[error("Context overflow: token budget exceeded")]
    ContextOverflow,

    #[error("Streaming error: {0}")]
    StreamError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Timeout after {0}s")]
    Timeout(u64),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Agent not initialized")]
    NotInitialized,

    #[error("Operation not supported by this adapter: {0}")]
    NotSupported(String),

    // LLM errors
    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Streaming error: {0}")]
    StreamingError(String),

    // Tool errors
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("File operation failed: {0}")]
    FileError(String),

    #[error("Pattern error: {0}")]
    PatternError(String),

    #[error("Edit failed: {0}")]
    EditFailed(String),

    #[error("Schema validation failed: {0}")]
    SchemaValidationError(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Command timed out after {0}s")]
    CommandTimeout(u64),

    // Generic errors
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Result type for nova operations
pub type NovaResult<T> = Result<T, NovaError>;

impl NovaError {
    /// Check if the error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            NovaError::LLMError(_)
                | NovaError::Timeout(_)
                | NovaError::StreamError(_)
                | NovaError::HttpError(_)
        )
    }

    /// Check if the error requires user intervention
    pub fn requires_user_action(&self) -> bool {
        matches!(
            self,
            NovaError::ApprovalRequired(_)
                | NovaError::ApprovalDenied(_)
                | NovaError::SecurityViolation(_)
        )
    }
}

impl From<mambalibs_http::client::HttpError> for NovaError {
    fn from(e: mambalibs_http::client::HttpError) -> Self {
        NovaError::HttpError(e.to_string())
    }
}
