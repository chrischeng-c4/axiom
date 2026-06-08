---
id: sdd-agents-review-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Agent-facing public interfaces are part of the AW Core client-independent workflow protocol surface."
---

# Review Agent Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/agents/review/types.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ReviewIssue` | projects/agentic-workflow/src/agents/review/types.rs | struct | pub | 40 |  |
| `ReviewType` | projects/agentic-workflow/src/agents/review/types.rs | enum | pub | 11 |  |
| `ReviewVerdict` | projects/agentic-workflow/src/agents/review/types.rs | enum | pub | 56 |  |
| `Severity` | projects/agentic-workflow/src/agents/review/types.rs | enum | pub | 20 |  |
| `is_approved` | projects/agentic-workflow/src/agents/review/types.rs | function | pub | 67 | is_approved(&self) -> bool |
| `is_needs_revision` | projects/agentic-workflow/src/agents/review/types.rs | function | pub | 71 | is_needs_revision(&self) -> bool |
| `is_rejected` | projects/agentic-workflow/src/agents/review/types.rs | function | pub | 75 | is_rejected(&self) -> bool |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ReviewType:
    type: string
    enum: [Spec, Code]
    description: What kind of artifact is being reviewed.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      serde_rename_all: snake_case

  Severity:
    type: string
    enum: [High, Medium, Low]
    description: Severity of a review issue.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      serde_rename_all: snake_case
    x-trait-impls:
      - trait: Display
        impl_mode: codegen
        dispatch:
          - { variant: High,   value: "High" }
          - { variant: Medium, value: "Medium" }
          - { variant: Low,    value: "Low" }

  ReviewIssue:
    type: object
    required: [severity, description, suggestion]
    description: A single issue found during review.
    properties:
      severity:
        $ref: "#/definitions/Severity"
        description: "Issue severity."
      description:
        type: string
        description: "Human-readable description of the issue."
      suggestion:
        type: string
        description: "Suggested fix or follow-up action."
      location:
        type: string
        description: "Optional spec/file location anchor."
        x-serde-skip-if: "Option::is_none"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ReviewVerdict:
    type: object
    description: >
      Verdict returned by a reviewer. Internally-tagged enum with three
      variants — Approved (unit), NeedsRevision { issues } (struct),
      Rejected { reason } (struct).
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_tag: verdict
      serde_rename_all: snake_case
      variants:
        - { name: Approved,    kind: unit, doc: "Reviewer approved the artifact." }
        - name: NeedsRevision
          kind: struct
          doc: "Reviewer flagged issues; artifact must be revised."
          fields:
            - { name: issues, rust_type: "Vec<ReviewIssue>" }
        - name: Rejected
          kind: struct
          doc: "Reviewer rejected the artifact outright."
          fields:
            - { name: reason, rust_type: "String" }
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/agents/review/types.rs -->
```rust
//! Review types used by [`ReviewAgent`] and [`CRRCycle`].

use serde::{Deserialize, Serialize};

/// What kind of artifact is being reviewed.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/types.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewType {
    Spec,
    Code,
}

/// Severity of a review issue.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/types.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    High,
    Medium,
    Low,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/types.md#schema.trait-impls.Display
impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::High => write!(f, "High"),
            Severity::Medium => write!(f, "Medium"),
            Severity::Low => write!(f, "Low"),
        }
    }
}

/// A single issue found during review.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    /// Issue severity.
    pub severity: Severity,
    /// Human-readable description of the issue.
    pub description: String,
    /// Suggested fix or follow-up action.
    pub suggestion: String,
    /// Optional spec/file location anchor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

/// Verdict returned by a reviewer. Internally-tagged enum with three variants — Approved (unit), NeedsRevision { issues } (struct), Rejected { reason } (struct).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum ReviewVerdict {
    /// Reviewer approved the artifact.
    Approved,
    /// Reviewer flagged issues; artifact must be revised.
    NeedsRevision { issues: Vec<ReviewIssue> },
    /// Reviewer rejected the artifact outright.
    Rejected { reason: String },
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/types.md#source
impl ReviewVerdict {
    pub fn is_approved(&self) -> bool {
        matches!(self, ReviewVerdict::Approved)
    }

    pub fn is_needs_revision(&self) -> bool {
        matches!(self, ReviewVerdict::NeedsRevision { .. })
    }

    pub fn is_rejected(&self) -> bool {
        matches!(self, ReviewVerdict::Rejected { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verdict_serializes_with_tag() {
        let v = ReviewVerdict::NeedsRevision {
            issues: vec![ReviewIssue {
                severity: Severity::High,
                description: "Missing section".to_string(),
                suggestion: "Add overview".to_string(),
                location: Some("spec.md:1".to_string()),
            }],
        };
        let json = serde_json::to_value(&v).unwrap();
        assert_eq!(json["verdict"], "needs_revision");
        assert_eq!(json["issues"][0]["severity"], "high");
    }

    #[test]
    fn test_verdict_approved_serializes() {
        let v = ReviewVerdict::Approved;
        let json = serde_json::to_value(&v).unwrap();
        assert_eq!(json["verdict"], "approved");
    }

    #[test]
    fn test_verdict_helpers() {
        assert!(ReviewVerdict::Approved.is_approved());
        assert!(!ReviewVerdict::Approved.is_needs_revision());
        assert!(ReviewVerdict::NeedsRevision { issues: vec![] }.is_needs_revision());
        assert!(ReviewVerdict::Rejected { reason: "x".into() }.is_rejected());
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::High), "High");
        assert_eq!(format!("{}", Severity::Medium), "Medium");
        assert_eq!(format!("{}", Severity::Low), "Low");
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/agents/review/types.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete review types module, including serde
      types, the Severity Display impl, ReviewVerdict helper methods, and unit
      tests.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```


# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Accurately describes the four shapes and the three patterns exercised. Hand-written boundary correctly enumerated (impl ReviewVerdict bool helpers + tests + use stmts).
- [schema] Correct: ReviewType + Severity get snake_case rename_all; Severity's Display dispatch lists PascalCase values explicitly; ReviewIssue uses x-serde-skip-if on `location`; ReviewVerdict combines `serde_tag: verdict` + `serde_rename_all: snake_case` as a single emitted attribute. Struct variants list fields with rust_type values that match the source verbatim.
- [changes] Two-entry split correctly partitions codegen (4 type decls + Display impl) from hand-written (impl ReviewVerdict bool helpers, tests, module preamble). `replaces:` lists all four type names.

## Review 2
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Promotes helper methods and tests into source-template ownership while leaving the language-neutral schema as the shape contract.
- [source] Uses `strip-managed-markers` to preserve existing Rust behavior and remove the mixed CODEGEN/HANDWRITE boundary.
- [changes] Correctly declares the file as regenerated from the `source` section.
