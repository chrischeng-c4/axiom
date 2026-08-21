//! SpecProtocol — domain contract for a specification file.
//!
//! Supersedes `SpecExcerpt` from `agents/restructure.rs`.  While
//! `SpecExcerpt` represents a ranked search result, `SpecProtocol`
//! is the full canonical contract used by agents that read or write
//! specification files (ChangeSpecAgent, CodebaseToSpecAgent).

use serde::{Deserialize, Serialize};

/// Authoring format of a specification file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecFormat {
    /// OpenRPC JSON — for MCP tool interface definitions.
    OpenRpc,
    /// JSON Schema — for data models and payloads.
    JsonSchema,
    /// Mermaid diagrams — state machines, flowcharts, sequence diagrams.
    Mermaid,
    /// YAML — config schemas, CLI command trees.
    Yaml,
    /// Markdown — tables, checklists, or mixed content.
    Markdown,
    /// Free-form prose (minimal; last resort).
    Prose,
}

impl Default for SpecFormat {
    fn default() -> Self {
        Self::Markdown
    }
}

impl std::fmt::Display for SpecFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SpecFormat::OpenRpc => "open_rpc",
            SpecFormat::JsonSchema => "json_schema",
            SpecFormat::Mermaid => "mermaid",
            SpecFormat::Yaml => "yaml",
            SpecFormat::Markdown => "markdown",
            SpecFormat::Prose => "prose",
        };
        write!(f, "{}", s)
    }
}

/// Domain contract for a specification file.
///
/// Used by `ChangeSpecAgent` and `CodebaseToSpecAgent`.  Consumers
/// map their ORM spec records to/from this type.
///
/// Supersedes: `SpecExcerpt` in `agents/restructure.rs`
/// (which is a search-result type, not a full domain contract).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecProtocol {
    /// Unique spec identifier (e.g. change-id + section slug).
    pub id: String,
    /// Repository-relative path to the spec file.
    pub path: String,
    /// Full text content of the spec.
    pub content: String,
    /// Authoring format.
    pub format: SpecFormat,
    /// Semver-style version string (e.g. `"1.0.0"`) or empty if unversioned.
    pub version: String,
    /// Optional URL to an external sync target (Confluence page, Google Doc, etc.).
    pub sync_target: Option<String>,
}

impl SpecProtocol {
    /// Create a minimal spec with required fields and defaults.
    pub fn new(id: impl Into<String>, path: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            content: content.into(),
            format: SpecFormat::default(),
            version: String::new(),
            sync_target: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_protocol_roundtrip() {
        let spec = SpecProtocol {
            id: "agent-protocols-spec".to_string(),
            path: ".aw/changes/agent-protocols/specs/agent-protocols-spec.md".to_string(),
            content: "# Agent Protocols Spec\n...".to_string(),
            format: SpecFormat::Markdown,
            version: "1.0.0".to_string(),
            sync_target: Some("https://confluence.example.com/pages/123".to_string()),
        };

        let json = serde_json::to_string(&spec).unwrap();
        let decoded: SpecProtocol = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.id, "agent-protocols-spec");
        assert_eq!(decoded.format, SpecFormat::Markdown);
        assert!(decoded.sync_target.is_some());
    }

    #[test]
    fn test_spec_format_display() {
        assert_eq!(SpecFormat::OpenRpc.to_string(), "open_rpc");
        assert_eq!(SpecFormat::JsonSchema.to_string(), "json_schema");
        assert_eq!(SpecFormat::Mermaid.to_string(), "mermaid");
        assert_eq!(SpecFormat::Markdown.to_string(), "markdown");
    }

    #[test]
    fn test_spec_protocol_new() {
        let spec = SpecProtocol::new("id-1", "path/to/spec.md", "content");
        assert_eq!(spec.format, SpecFormat::Markdown);
        assert!(spec.version.is_empty());
        assert!(spec.sync_target.is_none());
    }
}
