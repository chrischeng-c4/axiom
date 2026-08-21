//! CodeIndexProtocol — domain contract for a codebase module index entry.
//!
//! Used by `ReferenceCodebaseContextAgent` to communicate the structure
//! of an analysed module back to downstream agents.

use serde::{Deserialize, Serialize};

/// Domain contract for a single module's code index entry.
///
/// Used by `ReferenceCodebaseContextAgent`.  Consumers map their
/// codebase-index records to/from this type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIndexProtocol {
    /// Module path in the codebase (e.g. `src/agents/code_agent/mod.rs`
    /// or a dotted Python path like `cclab.agent.code_agent`).
    pub module_path: String,
    /// Public API endpoints or entry-point function signatures exported
    /// by this module.
    pub endpoints: Vec<String>,
    /// Domain model types (structs, enums, classes) defined in this module.
    pub models: Vec<String>,
    /// External crate / package dependencies declared by this module.
    pub dependencies: Vec<String>,
}

impl CodeIndexProtocol {
    /// Create a minimal code index entry.
    pub fn new(module_path: impl Into<String>) -> Self {
        Self {
            module_path: module_path.into(),
            endpoints: Vec::new(),
            models: Vec::new(),
            dependencies: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_index_protocol_roundtrip() {
        let entry = CodeIndexProtocol {
            module_path: "crates/cclab-agent/src/agents/code_agent/mod.rs".to_string(),
            endpoints: vec![
                "CodeAgent::run".to_string(),
                "CodeAgent::run_with_handler".to_string(),
            ],
            models: vec![
                "CodeAgent".to_string(),
                "CodeAgentConfig".to_string(),
                "ImplementationTask".to_string(),
            ],
            dependencies: vec!["async_trait".to_string(), "serde".to_string()],
        };

        let json = serde_json::to_string(&entry).unwrap();
        let decoded: CodeIndexProtocol = serde_json::from_str(&json).unwrap();

        assert_eq!(
            decoded.module_path,
            "crates/cclab-agent/src/agents/code_agent/mod.rs"
        );
        assert_eq!(decoded.endpoints.len(), 2);
        assert_eq!(decoded.models.len(), 3);
        assert_eq!(decoded.dependencies.len(), 2);
    }

    #[test]
    fn test_code_index_protocol_new() {
        let entry = CodeIndexProtocol::new("src/lib.rs");

        assert_eq!(entry.module_path, "src/lib.rs");
        assert!(entry.endpoints.is_empty());
        assert!(entry.models.is_empty());
        assert!(entry.dependencies.is_empty());
    }
}
