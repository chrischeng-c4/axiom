---
id: sdd-models-challenge
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Challenge

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/models/challenge.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Challenge` | projects/agentic-workflow/src/models/challenge.rs | struct | pub | 138 |  |
| `ChallengeImpact` | projects/agentic-workflow/src/models/challenge.rs | struct | pub | 124 |  |
| `ChallengeIssue` | projects/agentic-workflow/src/models/challenge.rs | struct | pub | 84 |  |
| `ChallengeVerdict` | projects/agentic-workflow/src/models/challenge.rs | enum | pub | 12 |  |
| `IssueSeverity` | projects/agentic-workflow/src/models/challenge.rs | enum | pub | 48 |  |
| `count_by_severity` | projects/agentic-workflow/src/models/challenge.rs | function | pub | 172 | count_by_severity(&self) -> (usize, usize, usize) |
| `emoji` | projects/agentic-workflow/src/models/challenge.rs | function | pub | 63 | emoji(&self) -> &'static str |
| `has_critical_issues` | projects/agentic-workflow/src/models/challenge.rs | function | pub | 192 | has_critical_issues(&self) -> bool |
| `is_approved` | projects/agentic-workflow/src/models/challenge.rs | function | pub | 30 | is_approved(&self) -> bool |
| `is_rejected` | projects/agentic-workflow/src/models/challenge.rs | function | pub | 40 | is_rejected(&self) -> bool |
| `name` | projects/agentic-workflow/src/models/challenge.rs | function | pub | 72 | name(&self) -> &'static str |
| `needs_revision` | projects/agentic-workflow/src/models/challenge.rs | function | pub | 35 | needs_revision(&self) -> bool |
| `new` | projects/agentic-workflow/src/models/challenge.rs | function | pub | 103 | new(         title: impl Into<String>,         severity: IssueSeverity,         location: impl Into<String>,         description: impl Into<String>,         suggestion: impl Into<String>,     ) -> Self |
| `new` | projects/agentic-workflow/src/models/challenge.rs | function | pub | 155 | new(change_id: impl Into<String>) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ChallengeVerdict:
    type: string
    enum:
      - Approved
      - NeedsRevision
      - Rejected
      - Unknown
    description: |
      Verdict from a challenge review.
      Approved: proposal is approved and ready for implementation.
      NeedsRevision: proposal needs revision to address issues.
      Rejected: proposal is rejected due to fundamental problems.
      Unknown: verdict could not be determined (parsing error or not filled).
    x-rust-enum:
      derive: [Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq]
      variants:
        - name: Approved
          doc: "Proposal is approved and ready for implementation."
        - name: NeedsRevision
          doc: "Proposal needs revision to address issues."
        - name: Rejected
          doc: "Proposal is rejected due to fundamental problems."
        - name: Unknown
          doc: "Verdict could not be determined (parsing error or not filled)."
    x-methods:
      - name: is_approved
        returns: bool
        impl_mode: codegen
        body: "matches!(self, Self::Approved)"
        doc: "True iff verdict is Approved."
      - name: needs_revision
        returns: bool
        impl_mode: codegen
        body: "matches!(self, Self::NeedsRevision)"
        doc: "True iff verdict is NeedsRevision."
      - name: is_rejected
        returns: bool
        impl_mode: codegen
        body: "matches!(self, Self::Rejected)"
        doc: "True iff verdict is Rejected."

  IssueSeverity:
    type: string
    enum:
      - Low
      - Medium
      - High
    description: |
      Severity level of a challenge issue. Variants are ordered Low < Medium < High via PartialOrd/Ord derives.
    x-rust-enum:
      derive: [Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord]
      variants:
        - name: Low
          doc: "Low severity issue."
        - name: Medium
          doc: "Medium severity issue."
        - name: High
          doc: "High severity issue."
    x-methods:
      - name: emoji
        returns: "&'static str"
        impl_mode: codegen
        doc: "Emoji symbol representing the severity level."
        dispatch:
          - variant: Low
            value: "🟢"
          - variant: Medium
            value: "🟡"
          - variant: High
            value: "🔴"
      - name: name
        returns: "&'static str"
        impl_mode: codegen
        doc: "Human-readable display name for the severity level."
        dispatch:
          - variant: Low
            value: "Low"
          - variant: Medium
            value: "Medium"
          - variant: High
            value: "High"

  ChallengeIssue:
    type: object
    required: [title, severity, location, description, suggestion]
    description: Represents an issue found during challenge.
    properties:
      title:
        type: string
        description: "Issue title."
      severity:
        $ref: "#/definitions/IssueSeverity"
        description: "Severity level of the issue."
      location:
        type: string
        description: "Location in proposal (e.g., 'proposal.md:15' or 'tasks.md:1.2')."
      description:
        type: string
        description: "Description of the issue."
      suggestion:
        type: string
        description: "Suggested fix."
      line_number:
        type: integer
        nullable: true
        description: "Line number in CHALLENGE.md (for reference). Optional; defaults to None."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
    x-constructor:
      name: new
      doc: "Create a new challenge issue."
      impl_mode: codegen
      args:
        - { name: title,       rust_type: "impl Into<String>", into: String }
        - { name: severity,    rust_type: IssueSeverity }
        - { name: location,    rust_type: "impl Into<String>", into: String }
        - { name: description, rust_type: "impl Into<String>", into: String }
        - { name: suggestion,  rust_type: "impl Into<String>", into: String }
      init:
        line_number: "None"

  ChallengeImpact:
    type: object
    required: [files_to_modify, new_files, tests_to_update, risk_level]
    description: Impact assessment for a challenge report.
    properties:
      files_to_modify:
        type: array
        items:
          type: string
          format: path
        description: "Files that need to be modified."
      new_files:
        type: array
        items:
          type: string
          format: path
        description: "New files that will be created."
      tests_to_update:
        type: array
        items:
          type: string
          format: path
        description: "Tests that need updates."
      risk_level:
        type: string
        description: "Overall risk level."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
      field_types:
        files_to_modify: "Vec<PathBuf>"
        new_files: "Vec<PathBuf>"
        tests_to_update: "Vec<PathBuf>"

  Challenge:
    type: object
    required: [change_id, generated_at, issues, validations_passed, recommendations, impact]
    description: Challenge report generated by Codex.
    properties:
      change_id:
        type: string
        description: "Change ID that was challenged."
      generated_at:
        type: string
        description: "When the challenge was generated (RFC 3339 timestamp)."
      issues:
        type: array
        items:
          $ref: "#/definitions/ChallengeIssue"
        description: "Issues found, sorted by severity."
      validations_passed:
        type: array
        items:
          type: string
        description: "Validations that passed."
      recommendations:
        type: array
        items:
          type: string
        description: "General recommendations."
      impact:
        $ref: "#/definitions/ChallengeImpact"
        description: "Impact assessment."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
    x-methods:
      - name: new
        returns: Self
        impl_mode: hand-written
        doc: |
          Create a new Challenge for the given change_id.
          Calls chrono::Local::now().to_rfc3339() for generated_at and initialises
          a nested ChallengeImpact struct literal — too complex for body literal codegen.
        args:
          - { name: change_id, rust_type: "impl Into<String>" }
      - name: count_by_severity
        returns: "(usize, usize, usize)"
        impl_mode: hand-written
        doc: |
          Count issues by severity. Returns (high, medium, low) tuple.
          Multi-branch iterator logic — hand-written.
      - name: has_critical_issues
        returns: bool
        impl_mode: hand-written
        doc: |
          True iff any issue has High severity.
          Iterator any() predicate — hand-written.
```
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/models/challenge.rs -->
```rust
use serde::{Deserialize, Serialize};

/// Verdict from a challenge review.
/// Approved: proposal is approved and ready for implementation.
/// NeedsRevision: proposal needs revision to address issues.
/// Rejected: proposal is rejected due to fundamental problems.
/// Unknown: verdict could not be determined (parsing error or not filled).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/challenge.md#schema
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChallengeVerdict {
    /// Proposal is approved and ready for implementation.
    #[serde(rename = "Approved")]
    Approved,
    /// Proposal needs revision to address issues.
    #[serde(rename = "NeedsRevision")]
    NeedsRevision,
    /// Proposal is rejected due to fundamental problems.
    #[serde(rename = "Rejected")]
    Rejected,
    /// Verdict could not be determined (parsing error or not filled).
    #[serde(rename = "Unknown")]
    Unknown,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/challenge.md#schema.impls
impl ChallengeVerdict {
    /// True iff verdict is Approved.
    pub fn is_approved(&self) -> bool {
        matches!(self, Self::Approved)
    }

    /// True iff verdict is NeedsRevision.
    pub fn needs_revision(&self) -> bool {
        matches!(self, Self::NeedsRevision)
    }

    /// True iff verdict is Rejected.
    pub fn is_rejected(&self) -> bool {
        matches!(self, Self::Rejected)
    }
}

/// Severity level of a challenge issue. Variants are ordered Low < Medium < High via PartialOrd/Ord derives.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/challenge.md#schema
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    /// Low severity issue.
    #[serde(rename = "Low")]
    Low,
    /// Medium severity issue.
    #[serde(rename = "Medium")]
    Medium,
    /// High severity issue.
    #[serde(rename = "High")]
    High,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/challenge.md#schema.impls
impl IssueSeverity {
    /// Emoji symbol representing the severity level.
    pub fn emoji(&self) -> &'static str {
        match self {
            IssueSeverity::Low => "🟢",
            IssueSeverity::Medium => "🟡",
            IssueSeverity::High => "🔴",
        }
    }

    /// Human-readable display name for the severity level.
    pub fn name(&self) -> &'static str {
        match self {
            IssueSeverity::Low => "Low",
            IssueSeverity::Medium => "Medium",
            IssueSeverity::High => "High",
        }
    }
}

/// Represents an issue found during challenge.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/challenge.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeIssue {
    /// Issue title.
    pub title: String,
    /// Severity level of the issue.
    pub severity: IssueSeverity,
    /// Location in proposal (e.g., 'proposal.md:15' or 'tasks.md:1.2').
    pub location: String,
    /// Description of the issue.
    pub description: String,
    /// Suggested fix.
    pub suggestion: String,
    /// Line number in CHALLENGE.md (for reference). Optional; defaults to None.
    #[serde(default)]
    pub line_number: Option<i64>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/challenge.md#schema.impls
impl ChallengeIssue {
    /// Create a new challenge issue.
    pub fn new(
        title: impl Into<String>,
        severity: IssueSeverity,
        location: impl Into<String>,
        description: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            severity,
            location: location.into(),
            description: description.into(),
            suggestion: suggestion.into(),
            line_number: None,
        }
    }
}

/// Impact assessment for a challenge report.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/challenge.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeImpact {
    /// Files that need to be modified.
    pub files_to_modify: Vec<String>,
    /// New files that will be created.
    pub new_files: Vec<String>,
    /// Tests that need updates.
    pub tests_to_update: Vec<String>,
    /// Overall risk level.
    pub risk_level: String,
}

/// Challenge report generated by Codex.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/challenge.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    /// Change ID that was challenged.
    pub change_id: String,
    /// When the challenge was generated (RFC 3339 timestamp).
    pub generated_at: String,
    /// Issues found, sorted by severity.
    pub issues: Vec<ChallengeIssue>,
    /// Validations that passed.
    pub validations_passed: Vec<String>,
    /// General recommendations.
    pub recommendations: Vec<String>,
    /// Impact assessment.
    pub impact: ChallengeImpact,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/challenge.md#source
impl Challenge {
    pub fn new(change_id: impl Into<String>) -> Self {
        Self {
            change_id: change_id.into(),
            generated_at: chrono::Local::now().to_rfc3339(),
            issues: Vec::new(),
            validations_passed: Vec::new(),
            recommendations: Vec::new(),
            impact: ChallengeImpact {
                files_to_modify: Vec::new(),
                new_files: Vec::new(),
                tests_to_update: Vec::new(),
                risk_level: "Unknown".to_string(),
            },
        }
    }

    /// Count issues by severity
    pub fn count_by_severity(&self) -> (usize, usize, usize) {
        let high = self
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::High)
            .count();
        let medium = self
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Medium)
            .count();
        let low = self
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Low)
            .count();
        (high, medium, low)
    }

    /// Check if there are critical issues (High severity)
    pub fn has_critical_issues(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == IssueSeverity::High)
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/models/challenge.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete challenge model module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [changes] `replaces:` correctly includes `"impl ChallengeVerdict"` (all 3 bool methods are codegen), `"impl IssueSeverity"` (both string-dispatch methods are codegen), and `"impl ChallengeIssue"` (only `::new`, which is codegen), while correctly omitting `"impl Challenge"` (all three methods — `new`, `count_by_severity`, `has_critical_issues` — are hand-written). The `Challenge` struct declaration is listed for replacement without its impl block, satisfying the critical scenario.rs lesson.
- [schema] `ChallengeVerdict` bool-body dispatch methods correctly use `body: "matches!(self, Self::<Variant>)"` literals with `impl_mode: codegen` — validating the bool-dispatch codegen path. All three variants covered; no spurious method for `Unknown` is added.
- [schema] `ChallengeImpact` correctly carries no `x-constructor` and no `x-methods`, consistent with the struct having no dispatch logic and being initialised entirely by the hand-written `Challenge::new`.
- [overview] Overview accurately summarises which items enter CODEGEN-BEGIN/CODEGEN-END and that `impl Challenge` remains outside — consistent with the `changes` section.
