// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models_preamble_source.md#source
// CODEGEN-BEGIN
//! Data types for spec alignment checking.
//!
//! Corresponds to the JSON Schema definitions in the check-alignment change spec:
//! SpecDocument, SpecSection, CodeBlock, Violation, ViolationKind, FileResult, CheckResult.
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Aggregate result from `spec_alignment::check()`.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Per-file results.
    pub files: Vec<FileResult>,
    /// Total violation count across all files.
    pub total_violations: usize,
    /// True if no violations and no uncovered requirements.
    pub passed: bool,
    /// Coverage report (present when check_with_coverage is used).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<CoverageReport>,
}

/// A fenced code block within a section.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    /// Code fence language (json, yaml, mermaid, etc.).
    pub lang: String,
    /// Line number of opening fence (1-based).
    pub line: usize,
    /// Raw content between fences.
    pub content: String,
    /// Parsed JSON value if lang=json and content is valid JSON.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parsed_json: Option<serde_json::Value>,
}

/// A single requirement's coverage status.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageEntry {
    /// Requirement ID.
    pub requirement_id: String,
    /// Spec file path.
    pub spec_path: String,
    /// Coverage status: `covered` or `uncovered`.
    pub status: String,
    /// @spec annotations matching this requirement.
    pub annotations: Vec<SpecAnnotation>,
}

/// Coverage analysis report.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    /// Requirements with matching annotations.
    pub covered: Vec<CoverageEntry>,
    /// Requirements with no matching annotations.
    pub uncovered_requirements: Vec<CoverageEntry>,
    /// Public fns without `@spec`.
    pub unspecced_functions: Vec<UnspeccedFunction>,
    /// Annotations pointing to non-existent paths.
    pub stale_annotations: Vec<SpecAnnotation>,
    /// Requirements not referenced by scenarios.
    pub orphan_requirements: Vec<OrphanRequirementEntry>,
    /// Schema/struct property mismatches.
    pub schema_struct_mismatches: Vec<SchemaStructMismatchEntry>,
    /// Ratio of covered requirements (0.0–1.0).
    pub coverage_ratio: f64,
}

/// Check result for a single file.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    /// File path.
    pub path: String,
    /// Status: `ok` or `fail`.
    pub status: String,
    /// Violations found.
    pub violations: Vec<Violation>,
}

/// A requirement in the Requirements table not referenced by any scenario.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanRequirementEntry {
    /// Requirement ID.
    pub requirement_id: String,
    /// Spec file path.
    pub spec_path: String,
    /// Description from the requirements table.
    pub description: Option<String>,
}

/// A mismatch between JSON Schema properties and Rust struct fields.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaStructMismatchEntry {
    /// Schema/struct name.
    pub schema_name: String,
    /// Field name.
    pub field: String,
    /// Mismatch kind.
    pub kind: String,
    /// Spec file path.
    pub spec_path: String,
}

/// Section type annotation parsed from legacy or attr-style comments.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionAnnotation {
    /// Declared section type (e.g. overview, config, logic).
    pub section_type: String,
    /// Declared lang (e.g. markdown, json, mermaid, yaml).
    pub lang: String,
    /// Optional attr-style metadata excluding core type/lang keys.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub attributes: std::collections::BTreeMap<String, String>,
}

/// A `@spec {path}#{id}` annotation found in source code.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecAnnotation {
    /// Spec file path referenced.
    pub spec_path: String,
    /// Requirement ID (e.g. `R1`).
    pub requirement_id: String,
    /// Source file where the annotation was found.
    pub source_file: String,
    /// Line number (1-based).
    pub line: usize,
    /// Comment syntax (`//`, `#`, `--`, `<!--`, `/*`).
    pub comment_syntax: String,
}

/// Parsed representation of a spec `.md` file.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecDocument {
    /// File path (relative to project root).
    pub path: String,
    /// Parsed YAML frontmatter.
    pub frontmatter: serde_json::Value,
    /// Parsed sections.
    pub sections: Vec<SpecSection>,
}

/// A single section parsed from heading + annotation + content.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecSection {
    /// Heading text (without `##` prefix).
    pub heading: String,
    /// Line number of the `## Heading` (1-based).
    pub line: usize,
    /// Section type annotation, if present.
    pub annotation: Option<SectionAnnotation>,
    /// Fenced code blocks found within this section.
    pub code_blocks: Vec<CodeBlock>,
    /// Raw body text trimmed of surrounding whitespace.
    #[serde(default)]
    pub body: String,
}

/// A public function without a `@spec` annotation.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnspeccedFunction {
    /// Function name.
    pub name: String,
    /// Symbol kind.
    pub kind: String,
    /// Source file path.
    pub source_file: String,
    /// Line number (1-based).
    pub line: usize,
}

/// A single validation violation.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// Violation kind.
    pub kind: ViolationKind,
    /// Human-readable violation message.
    pub message: String,
    /// Section heading (for format rules).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,
    /// Primary line number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// Multiple line numbers (for duplicates).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<Vec<usize>>,
    /// Definition name (for logical rules).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Expected code fence lang.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_lang: Option<String>,
    /// Field name (for schema/field conflicts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// Additional context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Violation kinds emitted by spec alignment checking.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationKind {
    MissingSectionAnnotation,
    DuplicateSection,
    FormatPriorityViolation,
    DuplicateDefinition,
    DefinitionConflictRequired,
    DefinitionConflictFieldName,
    DefinitionConflictSchema,
    RpcFieldConsistency,
    IoError,
    OrphanRequirement,
    NestedSchemaConflictRequired,
    NestedSchemaConflictSchema,
    NestedSchemaConflictFieldName,
    SchemaStructMismatch,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models_impl_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models_impl_source.md#source
impl ViolationKind {
    /// Returns true if this is a Phase 1 format/logical violation.
    ///
    /// Phase 1 violations are structural problems in spec files:
    /// missing annotations, duplicates, format priority issues, definition
    /// conflicts, and RPC field consistency issues.
    ///
    /// Phase 2 violations (OrphanRequirement) and non-spec issues (IoError,
    /// SchemaStructMismatch) return false.
    pub fn is_format_violation(&self) -> bool {
        matches!(
            self,
            ViolationKind::MissingSectionAnnotation
                | ViolationKind::DuplicateSection
                | ViolationKind::FormatPriorityViolation
                | ViolationKind::DuplicateDefinition
                | ViolationKind::DefinitionConflictRequired
                | ViolationKind::DefinitionConflictFieldName
                | ViolationKind::DefinitionConflictSchema
                | ViolationKind::RpcFieldConsistency
                | ViolationKind::NestedSchemaConflictRequired
                | ViolationKind::NestedSchemaConflictSchema
                | ViolationKind::NestedSchemaConflictFieldName
        )
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/models_impl_source.md#source
impl std::fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Serialize to JSON string to get the snake_case name from serde rename_all
        let json_str = serde_json::to_string(self).unwrap_or_default();
        // Strip surrounding quotes: "\"missing_section_annotation\"" -> "missing_section_annotation"
        let name = json_str.trim_matches('"');
        write!(f, "{}", name)
    }
}

// ─── Phase 2 types ──────────────────────────────────────────────────────────
// CODEGEN-END
