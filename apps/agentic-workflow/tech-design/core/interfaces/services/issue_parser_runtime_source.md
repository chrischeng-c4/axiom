---
id: sdd-interfaces-services-issue-parser-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Service interfaces expose AW Core project, issue, and platform boundary behavior to clients."
---

# Issue Parser Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/services/issue_parser.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AcceptanceCriterion` | apps/agentic-workflow/src/services/issue_parser.rs | struct | pub | 24 |  |
| `Decision` | apps/agentic-workflow/src/services/issue_parser.rs | struct | pub | 34 |  |
| `IssueQualityResult` | apps/agentic-workflow/src/services/issue_parser.rs | struct | pub | 44 |  |
| `IssueReferenceContext` | apps/agentic-workflow/src/services/issue_parser.rs | struct | pub | 54 |  |
| `IssueScope` | apps/agentic-workflow/src/services/issue_parser.rs | struct | pub | 66 |  |
| `Requirement` | apps/agentic-workflow/src/services/issue_parser.rs | struct | pub | 76 |  |
| `SpecPlanEntry` | apps/agentic-workflow/src/services/issue_parser.rs | struct | pub | 88 |  |
| `SpecReference` | apps/agentic-workflow/src/services/issue_parser.rs | struct | pub | 102 |  |
| `StructuredIssue` | apps/agentic-workflow/src/services/issue_parser.rs | struct | pub | 114 |  |
| `ValidationError` | apps/agentic-workflow/src/services/issue_parser.rs | struct | pub | 132 |  |
| `check_issue_body_section_format` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 160 | check_issue_body_section_format(     path_label: &std::path::Path,     body: &str, ) -> Vec<crate::validate::Finding> |
| `extract_issue_slug` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 714 | extract_issue_slug(description: &str) -> Option<String> |
| `find_slug_by_uuid_prefix` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 819 | find_slug_by_uuid_prefix(     project_root: &std::path::Path,     prefix: &str, ) -> anyhow::Result<Option<String>> |
| `generate_post_clarifications_md` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 1079 | generate_post_clarifications_md(     change_id: &str,     group_id: &str,     scope: &IssueScope,     acceptance_criteria: &[AcceptanceCriterion], ) -> String |
| `generate_pre_clarifications_md` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 1042 | generate_pre_clarifications_md(     change_id: &str,     group_id: &str,     decisions: &[Decision], ) -> String |
| `generate_reference_context_md` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 1122 | generate_reference_context_md(     change_id: &str,     group_id: &str,     ref_ctx: &IssueReferenceContext, ) -> String |
| `generate_requirements_md` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 1004 | generate_requirements_md(     change_id: &str,     group_id: &str,     structured: &StructuredIssue, ) -> String |
| `is_structured_issue` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 182 | is_structured_issue(body: &str) -> bool |
| `load_issue_body` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 963 | load_issue_body(project_root: &std::path::Path, slug: &str) -> Option<String> |
| `load_issue_title` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 932 | load_issue_title(project_root: &std::path::Path, slug: &str) -> Option<String> |
| `looks_like_uuid_prefix` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 882 | looks_like_uuid_prefix(s: &str) -> bool |
| `new` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 201 | new(error: impl Into<String>, missing: Vec<String>) -> Self |
| `parse_structured_issue` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 427 | parse_structured_issue(body: &str) -> Option<StructuredIssue> |
| `resolve_issue_slug` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 748 | resolve_issue_slug(     project_root: &std::path::Path,     description: &str,     issues: Option<&[String]>, ) -> Option<String> |
| `validate_issue_quality` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 338 | validate_issue_quality(body: &str) -> IssueQualityResult |
| `validate_structured_issue` | apps/agentic-workflow/src/services/issue_parser.rs | function | pub | 242 | validate_structured_issue(body: &str, state: IssueState) -> Result<(), ValidationError> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap issue-parser-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/services/issue_parser.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:issue-parser-runtime>"
    description: "Source template owns issue parser runtime behavior and tests."
```
