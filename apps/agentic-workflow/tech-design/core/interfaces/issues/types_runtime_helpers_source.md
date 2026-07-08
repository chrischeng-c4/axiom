---
id: sdd-interfaces-issues-types-runtime-helpers-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# Issue Types Runtime Helpers Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/issues/types.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Issue` | apps/agentic-workflow/src/issues/types.rs | struct | pub | 17 |  |
| `IssueErrorCode` | apps/agentic-workflow/src/issues/types.rs | enum | pub | 123 |  |
| `IssueFilter` | apps/agentic-workflow/src/issues/types.rs | struct | pub | 135 |  |
| `IssuePatch` | apps/agentic-workflow/src/issues/types.rs | struct | pub | 149 |  |
| `IssuePhase` | apps/agentic-workflow/src/issues/types.rs | enum | pub | 208 |  |
| `IssueSection` | apps/agentic-workflow/src/issues/types.rs | enum | pub | 229 |  |
| `IssueState` | apps/agentic-workflow/src/issues/types.rs | enum | pub | 244 |  |
| `IssueType` | apps/agentic-workflow/src/issues/types.rs | enum | pub | 257 |  |
| `ShipStatus` | apps/agentic-workflow/src/issues/types.rs | enum | pub | 274 |  |
| `apply` | apps/agentic-workflow/src/issues/types.rs | function | pub | 601 | apply(&self, issue: &mut Issue) |
| `as_str` | apps/agentic-workflow/src/issues/types.rs | function | pub | 421 | as_str(&self) -> &'static str |
| `as_str` | apps/agentic-workflow/src/issues/types.rs | function | pub | 451 | as_str(&self) -> &'static str |
| `as_str` | apps/agentic-workflow/src/issues/types.rs | function | pub | 495 | as_str(&self) -> &'static str |
| `as_str` | apps/agentic-workflow/src/issues/types.rs | function | pub | 516 | as_str(&self) -> &'static str |
| `as_str` | apps/agentic-workflow/src/issues/types.rs | function | pub | 720 | as_str(&self) -> &'static str |
| `default_slug` | apps/agentic-workflow/src/issues/types.rs | function | pub | 749 | default_slug(&self) -> String |
| `exit_code` | apps/agentic-workflow/src/issues/types.rs | function | pub | 729 | exit_code(&self) -> i32 |
| `from_labels` | apps/agentic-workflow/src/issues/types.rs | function | pub | 549 | from_labels(labels: &[String]) -> Self |
| `heading` | apps/agentic-workflow/src/issues/types.rs | function | pub | 461 | heading(&self) -> &'static str |
| `lifecycle_trailer` | apps/agentic-workflow/src/issues/types.rs | module | pub | 365 |  |
| `matches` | apps/agentic-workflow/src/issues/types.rs | function | pub | 566 | matches(&self, issue: &Issue) -> bool |
| `parse` | apps/agentic-workflow/src/issues/types.rs | function | pub | 433 | parse(s: &str) -> Option<Self> |
| `parse` | apps/agentic-workflow/src/issues/types.rs | function | pub | 480 | parse(s: &str) -> Option<Self> |
| `parse` | apps/agentic-workflow/src/issues/types.rs | function | pub | 526 | parse(s: &str) -> Option<Self> |
| `parse_loose` | apps/agentic-workflow/src/issues/types.rs | function | pub | 504 | parse_loose(s: &str) -> Option<Self> |
| `tag_name` | apps/agentic-workflow/src/issues/types.rs | function | pub | 471 | tag_name(&self) -> &'static str |
| `td_phase` | apps/agentic-workflow/src/issues/types.rs | module | pub | 298 |  |
| `workflow_role` | apps/agentic-workflow/src/issues/types.rs | function | pub | 538 | workflow_role(&self) -> &'static str |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap issue-types-runtime-helpers -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/issues/types.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:issue-types-runtime-helpers>"
    description: "Source template owns issue type runtime helper impls."
```
