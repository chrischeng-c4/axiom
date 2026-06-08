---
id: projects-score-src-project-rs
fill_sections: [changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Support CLI surfaces expose AW Core bootstrap, chat, hook, project, and workspace invariants."
---

# Standardized projects/agentic-workflow/src/cli/project.rs

## Overview
<!-- type: overview lang: markdown -->

Source TD for top-level AW health reports. `aw health` reports
production readiness, managed/semantic coverage, regenerability maturity, cb
and cold verification state, plus active WI projection locks so a pending
TD/CB gate is visible to operators and agents.

### Symbols

| Name | Target | Kind | Visibility | Signature |
|------|--------|------|------------|-----------|
| `ProjectHealthArgs` | projects/agentic-workflow/src/cli/project.rs | struct | pub | health command args |
| `ProjectHealthReport` | projects/agentic-workflow/src/cli/project.rs | struct | pub | health JSON report |
| `ProjectHealthStatus` | projects/agentic-workflow/src/cli/project.rs | enum | pub | healthy/blocked |
| `build_health_report` | projects/agentic-workflow/src/cli/project.rs | function | pub | build_health_report(project) |
| `ProjectHealthReport::from_components` | projects/agentic-workflow/src/cli/project.rs | function | pub | aggregate coverage components |
| `run_health` | projects/agentic-workflow/src/cli/project.rs | function | pub | run health command |

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/project.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Include workflow_lock_count and WI projection blocker summaries in
      project health.
```
