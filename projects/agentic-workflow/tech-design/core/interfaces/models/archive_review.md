---
id: sdd-models-archive-review
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Archive Review

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/models/archive_review.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ArchiveIssueCategory` | projects/agentic-workflow/src/models/archive_review.rs | enum | pub | 56 |  |
| `ArchiveReview` | projects/agentic-workflow/src/models/archive_review.rs | struct | pub | 127 |  |
| `ArchiveReviewIssue` | projects/agentic-workflow/src/models/archive_review.rs | struct | pub | 95 |  |
| `ArchiveReviewVerdict` | projects/agentic-workflow/src/models/archive_review.rs | enum | pub | 15 |  |
| `blocks_archive` | projects/agentic-workflow/src/models/archive_review.rs | function | pub | 174 | blocks_archive(&self) -> bool |
| `count_by_severity` | projects/agentic-workflow/src/models/archive_review.rs | function | pub | 182 | count_by_severity(&self, severity: IssueSeverity) -> usize |
| `emoji` | projects/agentic-workflow/src/models/archive_review.rs | function | pub | 43 | emoji(&self) -> &'static str |
| `format` | projects/agentic-workflow/src/models/archive_review.rs | function | pub | 155 | format(&self) -> String |
| `format_summary` | projects/agentic-workflow/src/models/archive_review.rs | function | pub | 190 | format_summary(&self) -> String |
| `name` | projects/agentic-workflow/src/models/archive_review.rs | function | pub | 33 | name(&self) -> &'static str |
| `name` | projects/agentic-workflow/src/models/archive_review.rs | function | pub | 80 | name(&self) -> &'static str |
| `new` | projects/agentic-workflow/src/models/archive_review.rs | function | pub | 109 | new(         severity: IssueSeverity,         category: ArchiveIssueCategory,         spec_file: impl Into<String>,         description: impl Into<String>,     ) -> Self |
| `new` | projects/agentic-workflow/src/models/archive_review.rs | function | pub | 139 | new(         verdict: ArchiveReviewVerdict,         issues: Vec<ArchiveReviewIssue>,         summary: impl Into<String>,     ) -> Self |
| `passed` | projects/agentic-workflow/src/models/archive_review.rs | function | pub | 169 | passed(&self) -> bool |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ArchiveReviewVerdict:
    type: string
    enum:
      - Approved
      - Reviewed
      - Rejected
      - Unknown
    description: |
      Overall verdict from a Codex archive quality review.
      Approved: all checks passed, safe to archive.
      Reviewed: minor issues found, needs revision.
      Rejected: major issues, requires manual intervention.
      Unknown: could not parse verdict.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      variants:
        - name: Approved
          doc: "All checks passed, safe to archive."
        - name: Reviewed
          doc: "Minor issues found, needs revision."
        - name: Rejected
          doc: "Major issues, requires manual intervention."
        - name: Unknown
          doc: "Could not parse verdict."
    x-methods:
      - name: name
        returns: "&'static str"
        impl_mode: codegen
        doc: "Human-readable display name for the verdict."
        dispatch:
          - variant: Approved
            value: "APPROVED"
          - variant: Reviewed
            value: "REVIEWED"
          - variant: Rejected
            value: "REJECTED"
          - variant: Unknown
            value: "UNKNOWN"
      - name: emoji
        returns: "&'static str"
        impl_mode: codegen
        doc: "Emoji symbol representing the verdict."
        dispatch:
          - variant: Approved
            value: "✅"
          - variant: Reviewed
            value: "⚠️"
          - variant: Rejected
            value: "❌"
          - variant: Unknown
            value: "❓"

  ArchiveIssueCategory:
    type: string
    enum:
      - MissingContent
      - Hallucination
      - FormatError
      - ChangelogError
      - BrokenReference
      - Other
    description: |
      Category of issue found during archive review.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      variants:
        - name: MissingContent
          doc: "Missing content from delta."
        - name: Hallucination
          doc: "Extra content not in delta (hallucination)."
        - name: FormatError
          doc: "Format violation."
        - name: ChangelogError
          doc: "CHANGELOG inaccuracy."
        - name: BrokenReference
          doc: "Cross-reference broken."
        - name: Other
          doc: "Other issues."
    x-methods:
      - name: name
        returns: "&'static str"
        impl_mode: codegen
        doc: "Human-readable display name for the category."
        dispatch:
          - variant: MissingContent
            value: "Missing Content"
          - variant: Hallucination
            value: "Hallucination"
          - variant: FormatError
            value: "Format Error"
          - variant: ChangelogError
            value: "CHANGELOG Error"
          - variant: BrokenReference
            value: "Broken Reference"
          - variant: Other
            value: "Other"

  ArchiveReviewIssue:
    type: object
    required: [severity, category, spec_file, description]
    description: Issue found during archive review.
    properties:
      severity:
        $ref: "challenge#/definitions/IssueSeverity"
        description: "Severity of the issue. Imported from sibling challenge module; not redeclared here."
      category:
        $ref: "#/definitions/ArchiveIssueCategory"
        description: "Category of the issue."
      spec_file:
        type: string
        description: "Spec file where the issue was found."
      description:
        type: string
        description: "Human-readable description of the issue."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
    x-constructor:
      name: new
      doc: "Create a new archive review issue."
      impl_mode: codegen
      args:
        - { name: severity,    rust_type: IssueSeverity }
        - { name: category,    rust_type: ArchiveIssueCategory }
        - { name: spec_file,   rust_type: "impl Into<String>", into: String }
        - { name: description, rust_type: "impl Into<String>", into: String }
    x-methods:
      - name: format
        returns: String
        impl_mode: hand-written
        doc: |
          Format issue for display. Layout:
          [<severity.name()>] <spec_file>: <category.name()> - <description>
          Non-trivial string formatting — hand-written.

  ArchiveReview:
    type: object
    required: [verdict, issues, summary]
    description: Result of a Codex archive quality review.
    properties:
      verdict:
        $ref: "#/definitions/ArchiveReviewVerdict"
        description: "Overall verdict."
      issues:
        type: array
        items:
          $ref: "#/definitions/ArchiveReviewIssue"
        description: "List of issues found during review."
      summary:
        type: string
        description: "Summary of review findings."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
    x-constructor:
      name: new
      doc: "Create a new archive review result."
      impl_mode: codegen
      args:
        - { name: verdict, rust_type: ArchiveReviewVerdict }
        - { name: issues,  rust_type: "Vec<ArchiveReviewIssue>" }
        - { name: summary, rust_type: "impl Into<String>", into: String }
    x-methods:
      - name: passed
        returns: bool
        impl_mode: hand-written
        doc: "True iff verdict is Approved."
      - name: blocks_archive
        returns: bool
        impl_mode: hand-written
        doc: "True iff verdict is Reviewed or Rejected (uses matches! macro — hand-written)."
      - name: count_by_severity
        returns: usize
        impl_mode: hand-written
        doc: "Count issues matching the given severity."
        args:
          - { name: severity, rust_type: IssueSeverity }
      - name: format_summary
        returns: String
        impl_mode: hand-written
        doc: |
          Format full review summary for display. Multi-line layout with verdict emoji/name,
          per-severity issue counts, and summary text — hand-written.
```
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/models/archive_review.rs -->
```rust
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#source
use super::IssueSeverity; // Re-use from challenge module

use serde::{Deserialize, Serialize};

/// Overall verdict from a Codex archive quality review.
/// Approved: all checks passed, safe to archive.
/// Reviewed: minor issues found, needs revision.
/// Rejected: major issues, requires manual intervention.
/// Unknown: could not parse verdict.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveReviewVerdict {
    /// All checks passed, safe to archive.
    #[serde(rename = "Approved")]
    Approved,
    /// Minor issues found, needs revision.
    #[serde(rename = "Reviewed")]
    Reviewed,
    /// Major issues, requires manual intervention.
    #[serde(rename = "Rejected")]
    Rejected,
    /// Could not parse verdict.
    #[serde(rename = "Unknown")]
    Unknown,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema.impls
impl ArchiveReviewVerdict {
    /// Human-readable display name for the verdict.
    pub fn name(&self) -> &'static str {
        match self {
            ArchiveReviewVerdict::Approved => "APPROVED",
            ArchiveReviewVerdict::Reviewed => "REVIEWED",
            ArchiveReviewVerdict::Rejected => "REJECTED",
            ArchiveReviewVerdict::Unknown => "UNKNOWN",
        }
    }

    /// Emoji symbol representing the verdict.
    pub fn emoji(&self) -> &'static str {
        match self {
            ArchiveReviewVerdict::Approved => "✅",
            ArchiveReviewVerdict::Reviewed => "⚠️",
            ArchiveReviewVerdict::Rejected => "❌",
            ArchiveReviewVerdict::Unknown => "❓",
        }
    }
}

/// Category of issue found during archive review.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveIssueCategory {
    /// Missing content from delta.
    #[serde(rename = "MissingContent")]
    MissingContent,
    /// Extra content not in delta (hallucination).
    #[serde(rename = "Hallucination")]
    Hallucination,
    /// Format violation.
    #[serde(rename = "FormatError")]
    FormatError,
    /// CHANGELOG inaccuracy.
    #[serde(rename = "ChangelogError")]
    ChangelogError,
    /// Cross-reference broken.
    #[serde(rename = "BrokenReference")]
    BrokenReference,
    /// Other issues.
    #[serde(rename = "Other")]
    Other,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema.impls
impl ArchiveIssueCategory {
    /// Human-readable display name for the category.
    pub fn name(&self) -> &'static str {
        match self {
            ArchiveIssueCategory::MissingContent => "Missing Content",
            ArchiveIssueCategory::Hallucination => "Hallucination",
            ArchiveIssueCategory::FormatError => "Format Error",
            ArchiveIssueCategory::ChangelogError => "CHANGELOG Error",
            ArchiveIssueCategory::BrokenReference => "Broken Reference",
            ArchiveIssueCategory::Other => "Other",
        }
    }
}

/// Issue found during archive review.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveReviewIssue {
    /// Severity of the issue. Imported from sibling challenge module; not redeclared here.
    pub severity: IssueSeverity,
    /// Category of the issue.
    pub category: ArchiveIssueCategory,
    /// Spec file where the issue was found.
    pub spec_file: String,
    /// Human-readable description of the issue.
    pub description: String,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema.impls
impl ArchiveReviewIssue {
    /// Create a new archive review issue.
    pub fn new(
        severity: IssueSeverity,
        category: ArchiveIssueCategory,
        spec_file: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            category,
            spec_file: spec_file.into(),
            description: description.into(),
        }
    }
}

/// Result of a Codex archive quality review.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveReview {
    /// Overall verdict.
    pub verdict: ArchiveReviewVerdict,
    /// List of issues found during review.
    pub issues: Vec<ArchiveReviewIssue>,
    /// Summary of review findings.
    pub summary: String,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema.impls
impl ArchiveReview {
    /// Create a new archive review result.
    pub fn new(
        verdict: ArchiveReviewVerdict,
        issues: Vec<ArchiveReviewIssue>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            verdict,
            issues,
            summary: summary.into(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#source
impl ArchiveReviewIssue {
    /// Format issue for display
    pub fn format(&self) -> String {
        format!(
            "[{}] {}: {} - {}",
            self.severity.name(),
            self.spec_file,
            self.category.name(),
            self.description
        )
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#source
impl ArchiveReview {
    /// Check if review passed (Approved verdict)
    pub fn passed(&self) -> bool {
        self.verdict == ArchiveReviewVerdict::Approved
    }

    /// Check if review should block archive
    pub fn blocks_archive(&self) -> bool {
        matches!(
            self.verdict,
            ArchiveReviewVerdict::Reviewed | ArchiveReviewVerdict::Rejected
        )
    }

    /// Count issues by severity
    pub fn count_by_severity(&self, severity: IssueSeverity) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == severity)
            .count()
    }

    /// Format review summary for display
    pub fn format_summary(&self) -> String {
        format!(
            r#"Archive Review: {} {}

Issues:
- High:   {}
- Medium: {}
- Low:    {}

Summary: {}
"#,
            self.verdict.emoji(),
            self.verdict.name(),
            self.count_by_severity(IssueSeverity::High),
            self.count_by_severity(IssueSeverity::Medium),
            self.count_by_severity(IssueSeverity::Low),
            self.summary
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verdict_display() {
        assert_eq!(ArchiveReviewVerdict::Approved.name(), "APPROVED");
        assert_eq!(ArchiveReviewVerdict::Approved.emoji(), "✅");
        assert_eq!(ArchiveReviewVerdict::Reviewed.name(), "REVIEWED");
        assert_eq!(ArchiveReviewVerdict::Reviewed.emoji(), "⚠️");
    }

    #[test]
    fn test_review_passed() {
        let review =
            ArchiveReview::new(ArchiveReviewVerdict::Approved, vec![], "All checks passed");
        assert!(review.passed());
        assert!(!review.blocks_archive());
    }

    #[test]
    fn test_review_blocks() {
        let review = ArchiveReview::new(
            ArchiveReviewVerdict::Reviewed,
            vec![ArchiveReviewIssue::new(
                IssueSeverity::High,
                ArchiveIssueCategory::MissingContent,
                "auth.md",
                "Missing R3",
            )],
            "Issues found",
        );
        assert!(!review.passed());
        assert!(review.blocks_archive());
        assert_eq!(review.count_by_severity(IssueSeverity::High), 1);
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/models/archive_review.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete archive review model module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [schema] Both enums (`ArchiveReviewVerdict`, `ArchiveIssueCategory`) declare correct `x-rust-enum.derive` arrays and complete `x-methods.dispatch` tables with all variants covered; all dispatch methods marked `impl_mode: codegen`.
- [schema] Both structs (`ArchiveReviewIssue`, `ArchiveReview`) declare `x-rust-struct.derive`, `x-constructor` with correct args (including `impl Into<String>` / `into: String` pattern), and all hand-written methods (`format`, `passed`, `blocks_archive`, `count_by_severity`, `format_summary`) marked `impl_mode: hand-written`.
- [schema] `IssueSeverity` cross-reference expressed via `$ref: "challenge#/definitions/IssueSeverity"` — generator will not redeclare it.
- [changes] `replaces:` lists the four type declarations plus their two standalone codegen `impl` blocks (`impl ArchiveReviewVerdict`, `impl ArchiveIssueCategory`); hand-written impl blocks (`impl ArchiveReviewIssue { format }`, `impl ArchiveReview { passed, ... }`) correctly absent from `replaces:`.
- [changes] Test-module preservation contract explicitly stated: `aw td gen-code` must not touch `#[cfg(test)] mod tests` on an `action: modify` file.
