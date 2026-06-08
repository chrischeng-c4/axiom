---
id: sdd-interfaces-issues-types-phase-namespaces-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# Issue Phase Namespace Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/issues/types.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Issue` | projects/agentic-workflow/src/issues/types.rs | struct | pub | 17 |  |
| `IssueErrorCode` | projects/agentic-workflow/src/issues/types.rs | enum | pub | 123 |  |
| `IssueFilter` | projects/agentic-workflow/src/issues/types.rs | struct | pub | 135 |  |
| `IssuePatch` | projects/agentic-workflow/src/issues/types.rs | struct | pub | 149 |  |
| `IssuePhase` | projects/agentic-workflow/src/issues/types.rs | enum | pub | 208 |  |
| `IssueSection` | projects/agentic-workflow/src/issues/types.rs | enum | pub | 229 |  |
| `IssueState` | projects/agentic-workflow/src/issues/types.rs | enum | pub | 244 |  |
| `IssueType` | projects/agentic-workflow/src/issues/types.rs | enum | pub | 257 |  |
| `ShipStatus` | projects/agentic-workflow/src/issues/types.rs | enum | pub | 274 |  |
| `apply` | projects/agentic-workflow/src/issues/types.rs | function | pub | 601 | apply(&self, issue: &mut Issue) |
| `as_str` | projects/agentic-workflow/src/issues/types.rs | function | pub | 421 | as_str(&self) -> &'static str |
| `as_str` | projects/agentic-workflow/src/issues/types.rs | function | pub | 451 | as_str(&self) -> &'static str |
| `as_str` | projects/agentic-workflow/src/issues/types.rs | function | pub | 495 | as_str(&self) -> &'static str |
| `as_str` | projects/agentic-workflow/src/issues/types.rs | function | pub | 516 | as_str(&self) -> &'static str |
| `as_str` | projects/agentic-workflow/src/issues/types.rs | function | pub | 720 | as_str(&self) -> &'static str |
| `default_slug` | projects/agentic-workflow/src/issues/types.rs | function | pub | 749 | default_slug(&self) -> String |
| `exit_code` | projects/agentic-workflow/src/issues/types.rs | function | pub | 729 | exit_code(&self) -> i32 |
| `from_labels` | projects/agentic-workflow/src/issues/types.rs | function | pub | 549 | from_labels(labels: &[String]) -> Self |
| `heading` | projects/agentic-workflow/src/issues/types.rs | function | pub | 461 | heading(&self) -> &'static str |
| `lifecycle_trailer` | projects/agentic-workflow/src/issues/types.rs | module | pub | 365 |  |
| `matches` | projects/agentic-workflow/src/issues/types.rs | function | pub | 566 | matches(&self, issue: &Issue) -> bool |
| `parse` | projects/agentic-workflow/src/issues/types.rs | function | pub | 433 | parse(s: &str) -> Option<Self> |
| `parse` | projects/agentic-workflow/src/issues/types.rs | function | pub | 480 | parse(s: &str) -> Option<Self> |
| `parse` | projects/agentic-workflow/src/issues/types.rs | function | pub | 526 | parse(s: &str) -> Option<Self> |
| `parse_loose` | projects/agentic-workflow/src/issues/types.rs | function | pub | 504 | parse_loose(s: &str) -> Option<Self> |
| `tag_name` | projects/agentic-workflow/src/issues/types.rs | function | pub | 471 | tag_name(&self) -> &'static str |
| `td_phase` | projects/agentic-workflow/src/issues/types.rs | module | pub | 298 |  |
| `workflow_role` | projects/agentic-workflow/src/issues/types.rs | function | pub | 538 | workflow_role(&self) -> &'static str |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap missing-generator:td-phase-enum -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/types.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:missing-generator:td-phase-enum>"
    description: "Source template owns TD phase and lifecycle trailer namespace helpers."
```
