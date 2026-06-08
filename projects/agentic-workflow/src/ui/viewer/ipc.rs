//! IPC types for viewer (legacy, kept for test compatibility)
//!
//! Note: The actual API handlers are now in mod.rs using axum.

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/ui/viewer/ipc_error.md#schema
// CODEGEN-BEGIN
/// Errors that can occur during IPC handling.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/ui/viewer/ipc_error.md#schema
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    #[error("Failed to parse IPC message: {0}")]
    ParseError(String),
    #[error("Annotation error: {0}")]
    AnnotationError(String),
    #[error("Viewer error: {0}")]
    ViewerError(#[from] super::manager::ViewerError),
}
// CODEGEN-END
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_error_display() {
        let err = IpcError::ParseError("test error".to_string());
        assert!(err.to_string().contains("test error"));
    }
}
