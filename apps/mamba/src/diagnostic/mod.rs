use crate::error::MambaError;
use crate::source::SourceMap;

/// Severity level for a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A diagnostic message with source location.
#[derive(Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub col: Option<u32>,
    pub source_line: Option<String>,
}

/// Render a MambaError as a human-readable diagnostic.
pub fn render_error(err: &MambaError, source_map: &SourceMap) -> String {
    if let Some(span) = err.span() {
        let file = source_map.get_file(span.file);
        let (line, col) = file.line_col(span.start);
        let src_line = file.line_text(line);
        format!(
            "error: {err}\n  --> {}:{line}:{col}\n   |\n{line:>3} | {src_line}\n   |",
            file.name,
        )
    } else {
        format!("error: {err}")
    }
}
