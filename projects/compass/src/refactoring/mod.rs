//! Refactoring engine with pluggable operation strategies
//!
//! Dispatches refactoring requests to the appropriate engine based on
//! the `RefactorKind`. Each engine implements the `RefactoringOp` trait.

mod extract;
mod extract_helpers;
mod inline;
mod move_def;
mod rename;
mod signature;
mod signature_helpers;

pub use extract::ExtractEngine;
pub use inline::InlineEngine;
pub use move_def::MoveDefEngine;
pub use rename::RenameEngine;
pub use signature::SignatureEngine;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::lens_error::{ArgusError, Result};
use crate::semantic::symbols::SymbolTable;
use crate::syntax::{Language, ParsedFile};
use crate::type_inference::{RefactorKind, RefactorRequest, RefactorResult};

// ============================================================================
// File context passed to every operation
// ============================================================================

/// Parsed context for a single file, shared across operations.
pub struct FileContext<'a> {
    /// Absolute path
    pub path: &'a PathBuf,
    /// Raw source text
    pub source: &'a str,
    /// tree-sitter parse result
    pub parsed: &'a ParsedFile,
    /// Symbol table built from the parsed file
    pub symbols: &'a SymbolTable,
    /// Detected language
    pub language: Language,
}

/// Multi-file project context for cross-file operations.
pub struct ProjectContext<'a> {
    /// All parsed files keyed by path
    pub files: &'a HashMap<PathBuf, (String, ParsedFile, SymbolTable)>,
}

// ============================================================================
// Trait every refactoring operation implements
// ============================================================================

/// A single refactoring operation.
pub trait RefactoringOp {
    /// Apply the operation and return edits.
    ///
    /// `file` is the primary file context.
    /// `project` provides cross-file data when available.
    fn apply(
        &self,
        request: &RefactorRequest,
        file: &FileContext<'_>,
        project: Option<&ProjectContext<'_>>,
    ) -> Result<RefactorResult>;
}

// ============================================================================
// Registry that dispatches to the right engine
// ============================================================================

/// Central dispatcher that maps `RefactorKind` to the correct engine.
pub struct RefactoringRegistry {
    rename: RenameEngine,
    extract: ExtractEngine,
    inline: InlineEngine,
    move_def: MoveDefEngine,
    signature: SignatureEngine,
}

impl RefactoringRegistry {
    pub fn new() -> Self {
        Self {
            rename: RenameEngine,
            extract: ExtractEngine,
            inline: InlineEngine,
            move_def: MoveDefEngine,
            signature: SignatureEngine,
        }
    }

    /// Dispatch a request to the appropriate engine.
    pub fn apply(
        &self,
        request: &RefactorRequest,
        file: &FileContext<'_>,
        project: Option<&ProjectContext<'_>>,
    ) -> Result<RefactorResult> {
        let engine: &dyn RefactoringOp = match &request.kind {
            RefactorKind::Rename { .. } => &self.rename,
            RefactorKind::ExtractFunction { .. }
            | RefactorKind::ExtractMethod { .. }
            | RefactorKind::ExtractVariable { .. } => &self.extract,
            RefactorKind::Inline => &self.inline,
            RefactorKind::MoveDefinition { .. } => &self.move_def,
            RefactorKind::ChangeSignature { .. } => &self.signature,
        };
        engine.apply(request, file, project)
    }
}

impl Default for RefactoringRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helpers shared across engines
// ============================================================================

/// Validate that `name` is a legal identifier for the given language.
pub(crate) fn validate_identifier(name: &str, language: Language) -> Result<()> {
    if name.is_empty() {
        return Err(ArgusError::invalid_identifier("name cannot be empty"));
    }
    let first = name.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return Err(ArgusError::invalid_identifier(format!(
            "'{}' must start with a letter or underscore",
            name
        )));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(ArgusError::invalid_identifier(format!(
            "'{}' contains invalid characters",
            name
        )));
    }
    if is_keyword(name, language) {
        return Err(ArgusError::invalid_identifier(format!(
            "'{}' is a reserved keyword",
            name
        )));
    }
    Ok(())
}

/// Check whether `name` is a reserved keyword in `language`.
pub(crate) fn is_keyword(name: &str, language: Language) -> bool {
    match language {
        Language::Python => matches!(
            name,
            "False"
                | "None"
                | "True"
                | "and"
                | "as"
                | "assert"
                | "async"
                | "await"
                | "break"
                | "class"
                | "continue"
                | "def"
                | "del"
                | "elif"
                | "else"
                | "except"
                | "finally"
                | "for"
                | "from"
                | "global"
                | "if"
                | "import"
                | "in"
                | "is"
                | "lambda"
                | "nonlocal"
                | "not"
                | "or"
                | "pass"
                | "raise"
                | "return"
                | "try"
                | "while"
                | "with"
                | "yield"
        ),
        Language::TypeScript | Language::JavaScript => matches!(
            name,
            "break"
                | "case"
                | "catch"
                | "class"
                | "const"
                | "continue"
                | "debugger"
                | "default"
                | "delete"
                | "do"
                | "else"
                | "enum"
                | "export"
                | "extends"
                | "false"
                | "finally"
                | "for"
                | "function"
                | "if"
                | "import"
                | "in"
                | "instanceof"
                | "new"
                | "null"
                | "return"
                | "super"
                | "switch"
                | "this"
                | "throw"
                | "true"
                | "try"
                | "typeof"
                | "var"
                | "void"
                | "while"
                | "with"
        ),
        Language::Rust => matches!(
            name,
            "as" | "break"
                | "const"
                | "continue"
                | "crate"
                | "else"
                | "enum"
                | "extern"
                | "false"
                | "fn"
                | "for"
                | "if"
                | "impl"
                | "in"
                | "let"
                | "loop"
                | "match"
                | "mod"
                | "move"
                | "mut"
                | "pub"
                | "ref"
                | "return"
                | "self"
                | "Self"
                | "static"
                | "struct"
                | "super"
                | "trait"
                | "true"
                | "type"
                | "unsafe"
                | "use"
                | "where"
                | "while"
                | "async"
                | "await"
                | "dyn"
        ),
        _ => false,
    }
}
