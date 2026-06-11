//! Diagnostic types (LSP-compatible)

use serde::{Deserialize, Serialize};

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

impl DiagnosticSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Information => "info",
            DiagnosticSeverity::Hint => "hint",
        }
    }
}

/// Diagnostic category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticCategory {
    Syntax,
    Type,
    Names,
    Logic,
    Security,
    Style,
    /// Diagnostics produced by user-defined custom rules (CUSTOM_* prefix)
    Custom,
}

impl DiagnosticCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticCategory::Syntax => "syntax",
            DiagnosticCategory::Type => "type",
            DiagnosticCategory::Names => "names",
            DiagnosticCategory::Logic => "logic",
            DiagnosticCategory::Security => "security",
            DiagnosticCategory::Style => "style",
            DiagnosticCategory::Custom => "custom",
        }
    }
}

/// Position in a text document (0-indexed)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// Range in a text document
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn from_node(node: &tree_sitter::Node<'_>) -> Self {
        let start = node.start_position();
        let end = node.end_position();
        Self {
            start: Position::new(start.row as u32, start.column as u32),
            end: Position::new(end.row as u32, end.column as u32),
        }
    }

    /// Check if a position is within this range
    pub fn contains(&self, line: u32, character: u32) -> bool {
        // Check if position is after start
        let after_start = line > self.start.line
            || (line == self.start.line && character >= self.start.character);

        // Check if position is before end
        let before_end =
            line < self.end.line || (line == self.end.line && character <= self.end.character);

        after_start && before_end
    }
}

/// Quick fix action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickFix {
    pub title: String,
    pub edits: Vec<TextEdit>,
}

/// Text edit for quick fixes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// A code diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub category: DiagnosticCategory,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub quick_fixes: Vec<QuickFix>,
}

impl Diagnostic {
    pub fn new(
        range: Range,
        severity: DiagnosticSeverity,
        code: impl Into<String>,
        category: DiagnosticCategory,
        message: impl Into<String>,
    ) -> Self {
        Self {
            range,
            severity,
            code: code.into(),
            category,
            message: message.into(),
            quick_fixes: Vec::new(),
        }
    }

    pub fn error(
        range: Range,
        code: impl Into<String>,
        category: DiagnosticCategory,
        message: impl Into<String>,
    ) -> Self {
        Self::new(range, DiagnosticSeverity::Error, code, category, message)
    }

    pub fn warning(
        range: Range,
        code: impl Into<String>,
        category: DiagnosticCategory,
        message: impl Into<String>,
    ) -> Self {
        Self::new(range, DiagnosticSeverity::Warning, code, category, message)
    }

    pub fn with_fix(mut self, title: impl Into<String>, edits: Vec<TextEdit>) -> Self {
        self.quick_fixes.push(QuickFix {
            title: title.into(),
            edits,
        });
        self
    }
}
