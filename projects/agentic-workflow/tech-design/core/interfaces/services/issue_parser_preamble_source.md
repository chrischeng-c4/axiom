---
id: sdd-interfaces-services-issue-parser-preamble-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Service interfaces expose AW Core project, issue, and platform boundary behavior to clients."
---

# Issue Parser Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/issue_parser.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AcceptanceCriterion` | projects/agentic-workflow/src/services/issue_parser.rs | struct | pub | 24 |  |
| `Decision` | projects/agentic-workflow/src/services/issue_parser.rs | struct | pub | 34 |  |
| `IssueQualityResult` | projects/agentic-workflow/src/services/issue_parser.rs | struct | pub | 44 |  |
| `IssueReferenceContext` | projects/agentic-workflow/src/services/issue_parser.rs | struct | pub | 54 |  |
| `IssueScope` | projects/agentic-workflow/src/services/issue_parser.rs | struct | pub | 66 |  |
| `Requirement` | projects/agentic-workflow/src/services/issue_parser.rs | struct | pub | 76 |  |
| `SpecPlanEntry` | projects/agentic-workflow/src/services/issue_parser.rs | struct | pub | 88 |  |
| `SpecReference` | projects/agentic-workflow/src/services/issue_parser.rs | struct | pub | 102 |  |
| `StructuredIssue` | projects/agentic-workflow/src/services/issue_parser.rs | struct | pub | 114 |  |
| `ValidationError` | projects/agentic-workflow/src/services/issue_parser.rs | struct | pub | 132 |  |
| `check_issue_body_section_format` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 160 | check_issue_body_section_format(     path_label: &std::path::Path,     body: &str, ) -> Vec<crate::validate::Finding> |
| `extract_issue_slug` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 714 | extract_issue_slug(description: &str) -> Option<String> |
| `find_slug_by_uuid_prefix` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 819 | find_slug_by_uuid_prefix(     project_root: &std::path::Path,     prefix: &str, ) -> anyhow::Result<Option<String>> |
| `generate_post_clarifications_md` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 1079 | generate_post_clarifications_md(     change_id: &str,     group_id: &str,     scope: &IssueScope,     acceptance_criteria: &[AcceptanceCriterion], ) -> String |
| `generate_pre_clarifications_md` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 1042 | generate_pre_clarifications_md(     change_id: &str,     group_id: &str,     decisions: &[Decision], ) -> String |
| `generate_reference_context_md` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 1122 | generate_reference_context_md(     change_id: &str,     group_id: &str,     ref_ctx: &IssueReferenceContext, ) -> String |
| `generate_requirements_md` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 1004 | generate_requirements_md(     change_id: &str,     group_id: &str,     structured: &StructuredIssue, ) -> String |
| `is_structured_issue` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 182 | is_structured_issue(body: &str) -> bool |
| `load_issue_body` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 963 | load_issue_body(project_root: &std::path::Path, slug: &str) -> Option<String> |
| `load_issue_title` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 932 | load_issue_title(project_root: &std::path::Path, slug: &str) -> Option<String> |
| `looks_like_uuid_prefix` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 882 | looks_like_uuid_prefix(s: &str) -> bool |
| `new` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 201 | new(error: impl Into<String>, missing: Vec<String>) -> Self |
| `parse_structured_issue` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 427 | parse_structured_issue(body: &str) -> Option<StructuredIssue> |
| `resolve_issue_slug` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 748 | resolve_issue_slug(     project_root: &std::path::Path,     description: &str,     issues: Option<&[String]>, ) -> Option<String> |
| `validate_issue_quality` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 338 | validate_issue_quality(body: &str) -> IssueQualityResult |
| `validate_structured_issue` | projects/agentic-workflow/src/services/issue_parser.rs | function | pub | 242 | validate_structured_issue(body: &str, state: IssueState) -> Result<(), ValidationError> |
## Source
<!-- type: source lang: rust -->

```rust
//! Issue section parser for structured issue format.
//!
//! Parses markdown bodies from temp-backed issue working-copy files
//! to extract structured sections that can skip early SDD phases.
//!
//! Required sections: `## Problem`, `## Requirements`, `## Scope`
//! Optional sections: `## Acceptance Criteria`, `## Key Decisions`, `## Reference Context`
//!
//! Write-time validation lives here as well: see [`validate_structured_issue`]
//! and [`ValidationError`]. Validation is tiered by the issue's `state`
//! (draft vs non-draft) per `projects/agentic-workflow/logic/structured-issue.md`.

use crate::issues::IssueState;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/issue_parser.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "<handwrite-gap:issue-parser-preamble>"
    description: "Source template owns issue parser documentation and non-serde import."
```
