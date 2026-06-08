//! Data types for the agent context builder.
//!
//! Defines the request, response, and entry types used throughout the context
//! building pipeline. All types derive `Serialize` and `Deserialize` for JSON
//! output (NF4).

use std::collections::HashMap;

// ============================================================================
// Request Types
// ============================================================================

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/context_builder/types.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// A single file entry in the context result.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/context_builder/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    pub path: String,
    pub reason: ContextReason,
    pub symbols: Vec<String>,
    pub depth: u32,
    /// Score is internal; never serialized.
    #[serde(skip_serializing)]
    pub score: f64,
}

/// Reason why a file appears in the context.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/context_builder/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContextReason {
    Target,
    ImportedByTarget,
    CalledByTarget,
    TransitiveDep,
    CallsTarget,
    TransitiveCaller,
    TestFile,
}

/// Request to build context for one or more targets.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/context_builder/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRequest {
    pub targets: Vec<ContextTarget>,
    #[serde(default = "default_depth")]
    pub depth: u32,
}

/// Full context response.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/context_builder/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextResponse {
    pub must_read: Vec<ContextEntry>,
    pub may_affect: Vec<ContextEntry>,
    pub type_context: HashMap<String, String>,
    pub stats: ContextStats,
}

/// Statistics about the context building process.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/context_builder/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStats {
    pub targets_resolved: usize,
    pub targets_unresolved: usize,
    pub files_scanned: usize,
    pub time_ms: u64,
}

/// A single target for context building (file:symbol).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/context_builder/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextTarget {
    pub file: String,
    pub symbol: String,
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/context_builder/types.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/interfaces/context_builder/types.md#source
impl ContextTarget {
    /// Parse a `file:symbol` string into a ContextTarget.
    ///
    /// Returns `None` if the string doesn't contain a `:` separator.
    pub fn parse(input: &str) -> Option<Self> {
        let (file, symbol) = input.rsplit_once(':')?;
        if file.is_empty() || symbol.is_empty() {
            return None;
        }
        Some(Self {
            file: file.to_string(),
            symbol: symbol.to_string(),
        })
    }
}

fn default_depth() -> u32 {
    2
}

// ============================================================================
// Response Types
// ============================================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/context_builder/types.md#source
impl ContextResponse {
    /// Create an empty response with zero stats.
    pub fn empty() -> Self {
        Self {
            must_read: Vec::new(),
            may_affect: Vec::new(),
            type_context: HashMap::new(),
            stats: ContextStats {
                targets_resolved: 0,
                targets_unresolved: 0,
                files_scanned: 0,
                time_ms: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_target_parse_valid() {
        let t = ContextTarget::parse("src/services/user.py:get_user").unwrap();
        assert_eq!(t.file, "src/services/user.py");
        assert_eq!(t.symbol, "get_user");
    }

    #[test]
    fn test_context_target_parse_colon_in_path() {
        let t = ContextTarget::parse("C:/src/user.py:get_user").unwrap();
        assert_eq!(t.file, "C:/src/user.py");
        assert_eq!(t.symbol, "get_user");
    }

    #[test]
    fn test_context_target_parse_no_colon() {
        assert!(ContextTarget::parse("not_valid_format").is_none());
    }

    #[test]
    fn test_context_target_parse_empty_file() {
        assert!(ContextTarget::parse(":symbol").is_none());
    }

    #[test]
    fn test_context_target_parse_empty_symbol() {
        assert!(ContextTarget::parse("file:").is_none());
    }

    #[test]
    fn test_context_target_parse_empty_string() {
        assert!(ContextTarget::parse("").is_none());
    }
}

// CODEGEN-END
