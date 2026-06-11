//! Serde-serializable types for agent output format.
//!
//! Symbol-centric JSON output optimized for LLM agent consumption.
//! Uses `skip_serializing_if` to omit empty fields for compactness (R9).

use serde::Serialize;
use std::collections::BTreeMap;

/// Top-level agent output: symbol-centric analysis result.
///
/// Required fields: `symbols`, `stats`.
/// Optional (omitted when empty): `imports`, `issues`, `impact`.
#[derive(Debug, Clone, Serialize)]
pub struct AgentOutput {
    /// Map of symbol qualified name to definition info.
    pub symbols: BTreeMap<String, SymbolDef>,

    /// Map of file path to list of imported symbol qualified names.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub imports: BTreeMap<String, Vec<String>>,

    /// Diagnostics attributed to nearest enclosing symbol.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<AgentIssue>,

    /// Map of symbol qualified name to list of "file:line" reference locations.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub impact: BTreeMap<String, Vec<String>>,

    /// Summary statistics.
    pub stats: AgentStats,
}

/// Definition info for a single symbol.
#[derive(Debug, Clone, Serialize)]
pub struct SymbolDef {
    /// Type signature string (e.g. "(int) -> User"). Omitted if unknown.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_sig: Option<String>,

    /// Relative file path from project root.
    pub file: String,

    /// 1-based line number.
    pub line: u32,

    /// Symbol kind: function, class, method, variable, constant, interface, type_alias, module.
    pub kind: String,
}

/// A diagnostic issue attributed to the nearest enclosing symbol.
#[derive(Debug, Clone, Serialize)]
pub struct AgentIssue {
    /// Severity: error, warning, info, hint.
    pub severity: String,

    /// Nearest enclosing symbol name, or "<file-level>" if none.
    pub symbol: String,

    /// File path.
    pub file: String,

    /// 1-based line number.
    pub line: u32,

    /// Diagnostic rule code (e.g. "PY101").
    pub code: String,

    /// Diagnostic message.
    pub message: String,
}

/// Summary statistics for the agent output.
#[derive(Debug, Clone, Serialize)]
pub struct AgentStats {
    pub files_checked: usize,
    pub symbols_found: usize,
    pub issues_count: usize,
    pub impact_edges: usize,
}
