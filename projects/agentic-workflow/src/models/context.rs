//! Context artifact data models for 3-stage exploration
//!
//! These structs represent the structured output of each exploration stage
//! in the decide-change v2 workflow. Each context artifact is an **index**
//! (what was scanned, what was found, where it lives) rather than a copy
//! of content.

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/models/context.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Codebase context data (legacy — now part of unified reference_context).
/// Captures codebase analysis using Lens tools. lens_tools_used proves
/// which analysis tools were invoked.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/context.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseContext {
    /// Lens tools that were used during codebase analysis.
    pub lens_tools_used: Vec<String>,
    /// Relevant codebase files found.
    pub files: Vec<FileRef>,
    /// Results from each Lens tool invocation.
    pub lens_results: Vec<LensResult>,
    /// Dependency graph edges as strings (defaults to empty list).
    #[serde(default)]
    pub dependency_graph: Vec<String>,
}

/// Context artifact type identifier.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/context.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextType {
    /// Spec context artifact (serialized as spec_context).
    SpecContext,
    /// Knowledge context artifact (serialized as knowledge_context).
    KnowledgeContext,
    /// Codebase context artifact (serialized as codebase_context).
    CodebaseContext,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/context.md#schema.impls
impl ContextType {
    /// Get the filename for this context type.
    pub fn filename(&self) -> &'static str {
        match self {
            ContextType::SpecContext => "spec_context.md",
            ContextType::KnowledgeContext => "knowledge_context.md",
            ContextType::CodebaseContext => "codebase_context.md",
        }
    }
}

/// Reference to a knowledge document with summary.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/context.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocRef {
    /// Document file path.
    pub path: String,
    /// Summary of the document's relevance.
    pub summary: String,
    /// Specific sections within the document that are relevant.
    #[serde(default)]
    pub relevant_sections: Vec<String>,
}

/// Reference to a codebase file with symbols and role.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/context.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRef {
    /// File path relative to workspace root.
    pub path: String,
    /// Symbols of interest within the file.
    #[serde(default)]
    pub symbols: Vec<String>,
    /// Role of this file in the change (defaults to empty string).
    #[serde(default)]
    pub role: String,
}

/// An inaccuracy found during review.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/context.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inaccuracy {
    /// Location of the inaccuracy (e.g. section name or field path).
    pub location: String,
    /// What the correct value or content should be.
    pub expected: String,
    /// What was actually found.
    pub actual: String,
}

/// Knowledge context data (legacy — now part of unified reference_context).
/// Captures analysis of knowledge base documents relevant to the change.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/context.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeContext {
    /// Knowledge categories that were scanned.
    pub scanned_categories: Vec<String>,
    /// Relevant knowledge documents found.
    pub docs: Vec<DocRef>,
    /// Relevant patterns found.
    pub patterns: Vec<PatternRef>,
    /// Pitfalls to be aware of (defaults to empty list).
    #[serde(default)]
    pub pitfalls: Vec<String>,
}

/// Result from a Lens tool invocation.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/context.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensResult {
    /// Name of the Lens tool invoked.
    pub tool: String,
    /// Query string passed to the tool.
    pub query: String,
    /// Summary of the tool result.
    pub summary: String,
}

/// A missing checklist item in review feedback.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/context.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingItem {
    /// The checklist item that is missing.
    pub checklist_item: String,
    /// Details explaining what is missing and why it matters.
    pub details: String,
}

/// Reference to a pattern found in the knowledge base.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/context.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRef {
    /// Pattern name.
    pub name: String,
    /// Source document where the pattern is described.
    pub source: String,
    /// Description of the pattern.
    pub description: String,
}

/// Structured feedback for REVIEWED verdict.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/context.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFeedback {
    /// The review verdict.
    pub verdict: ReviewVerdict,
    /// The workflow stage being reviewed.
    pub stage: String,
    /// Iteration number of the review (u32).
    pub iteration: u32,
    /// Path of the artifact file being reviewed.
    pub artifact_file: String,
    /// Missing checklist items found during review.
    #[serde(default)]
    pub missing_items: Vec<MissingItem>,
    /// Inaccuracies found during review.
    #[serde(default)]
    pub inaccuracies: Vec<Inaccuracy>,
}

/// Review verdict from agent self-review.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/context.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReviewVerdict {
    /// Artifact is approved with no required changes.
    Approved,
    /// Artifact is reviewed with feedback; revisions required.
    Reviewed,
    /// Artifact is rejected; significant rework required.
    Rejected,
}

/// Spec context data (legacy — now part of unified reference_context).
/// Captures analysis of existing main specs relevant to the change.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/context.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecContext {
    /// Spec groups that were scanned during exploration.
    pub scanned_groups: Vec<String>,
    /// Relevant spec references found.
    pub specs: Vec<SpecRef>,
    /// Spec dependency identifiers.
    pub dependencies: Vec<String>,
    /// Identified gaps in existing specs.
    pub gaps: Vec<String>,
}

/// Reference to an existing spec with relevance assessment.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/context.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecRef {
    /// Spec identifier.
    pub id: String,
    /// Spec group (defaults to empty string).
    #[serde(default)]
    pub group: String,
    /// Relevance level (e.g. high, medium, low).
    pub relevance: String,
    /// Why this spec is relevant (defaults to empty string).
    #[serde(default)]
    pub reason: String,
    /// Key sections within the spec that are relevant.
    #[serde(default)]
    pub key_sections: Vec<String>,
}
// CODEGEN-END
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_context_serialization() {
        let ctx = SpecContext {
            scanned_groups: vec!["sdd".to_string(), "cli".to_string()],
            specs: vec![SpecRef {
                id: "cli-architecture".to_string(),
                group: "cli".to_string(),
                relevance: "high".to_string(),
                reason: "Directly affected module".to_string(),
                key_sections: vec!["CLI Registration".to_string()],
            }],
            dependencies: vec!["sdd".to_string()],
            gaps: vec!["No existing spec for decide-change v2".to_string()],
        };
        let json = serde_json::to_string(&ctx).unwrap();
        let parsed: SpecContext = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.specs.len(), 1);
        assert_eq!(parsed.scanned_groups.len(), 2);
        assert_eq!(parsed.gaps.len(), 1);
    }

    #[test]
    fn test_knowledge_context_serialization() {
        let ctx = KnowledgeContext {
            scanned_categories: vec!["architecture".to_string()],
            docs: vec![DocRef {
                path: "00-architecture/01-overview.md".to_string(),
                summary: "Project architecture overview".to_string(),
                relevant_sections: vec!["MCP Tools".to_string()],
            }],
            patterns: vec![PatternRef {
                name: "MCP tool pattern".to_string(),
                source: "00-architecture/01-overview.md".to_string(),
                description: "Standard MCP tool definition + execute pattern".to_string(),
            }],
            pitfalls: vec!["Avoid mixing CLI and MCP".to_string()],
        };
        let json = serde_json::to_string(&ctx).unwrap();
        let parsed: KnowledgeContext = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.docs.len(), 1);
        assert_eq!(parsed.patterns.len(), 1);
        assert_eq!(parsed.scanned_categories.len(), 1);
    }

    #[test]
    fn test_codebase_context_serialization() {
        let ctx = CodebaseContext {
            lens_tools_used: vec!["lens_symbols".to_string(), "lens_references".to_string()],
            files: vec![FileRef {
                path: "src/mcp/tools/decide_change.rs".to_string(),
                symbols: vec!["DecideAction".to_string()],
                role: "MCP tool entry point".to_string(),
            }],
            lens_results: vec![LensResult {
                tool: "lens_symbols".to_string(),
                query: "DecideAction".to_string(),
                summary: "Found enum with 10 variants".to_string(),
            }],
            dependency_graph: vec!["decide_change.rs → context.rs".to_string()],
        };
        let json = serde_json::to_string(&ctx).unwrap();
        let parsed: CodebaseContext = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.files.len(), 1);
        assert_eq!(parsed.lens_results.len(), 1);
        assert_eq!(parsed.lens_tools_used.len(), 2);
    }

    #[test]
    fn test_review_verdict_serialization() {
        let v = ReviewVerdict::Reviewed;
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "\"REVIEWED\"");
    }

    #[test]
    fn test_context_type_filename() {
        assert_eq!(ContextType::SpecContext.filename(), "spec_context.md");
        assert_eq!(
            ContextType::KnowledgeContext.filename(),
            "knowledge_context.md"
        );
        assert_eq!(
            ContextType::CodebaseContext.filename(),
            "codebase_context.md"
        );
    }

    #[test]
    fn test_pattern_ref_serialization() {
        let p = PatternRef {
            name: "Service pattern".to_string(),
            source: "knowledge/patterns.md".to_string(),
            description: "Input struct + validation + rendering".to_string(),
        };
        let json = serde_json::to_string(&p).unwrap();
        let parsed: PatternRef = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Service pattern");
    }

    #[test]
    fn test_file_ref_serialization() {
        let f = FileRef {
            path: "src/lib.rs".to_string(),
            symbols: vec!["main".to_string()],
            role: "entry point".to_string(),
        };
        let json = serde_json::to_string(&f).unwrap();
        let parsed: FileRef = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.path, "src/lib.rs");
        assert_eq!(parsed.symbols, vec!["main"]);
    }

    #[test]
    fn test_lens_result_serialization() {
        let r = LensResult {
            tool: "lens_impact".to_string(),
            query: "context.rs".to_string(),
            summary: "3 direct dependents".to_string(),
        };
        let json = serde_json::to_string(&r).unwrap();
        let parsed: LensResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.tool, "lens_impact");
    }
}
