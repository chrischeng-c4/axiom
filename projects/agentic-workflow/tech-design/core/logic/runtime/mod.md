---
id: projects-sdd-src-runtime-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: root-envelope-completion-contract
    claim: root-envelope-completion-contract
    coverage: full
    rationale: "Runtime envelope and session logic define the root-runner completion and HITL contract."
---

# Standardized projects/agentic-workflow/src/runtime/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/runtime/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `envelope` | projects/agentic-workflow/src/runtime/mod.rs | module | pub | 9 |  |
| `event` | projects/agentic-workflow/src/runtime/mod.rs | module | pub | 10 |  |
| `github_backend` | projects/agentic-workflow/src/runtime/mod.rs | module | pub | 11 |  |
| `gitlab_backend` | projects/agentic-workflow/src/runtime/mod.rs | module | pub | 12 |  |
| `issue_backend` | projects/agentic-workflow/src/runtime/mod.rs | module | pub | 13 |  |
| `jira_backend` | projects/agentic-workflow/src/runtime/mod.rs | module | pub | 14 |  |
| `mainthread` | projects/agentic-workflow/src/runtime/mod.rs | module | pub | 15 |  |
| `router` | projects/agentic-workflow/src/runtime/mod.rs | module | pub | 16 |  |
| `score_process` | projects/agentic-workflow/src/runtime/mod.rs | module | pub | 17 |  |
| `session` | projects/agentic-workflow/src/runtime/mod.rs | module | pub | 18 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/runtime/mod.rs -->
```rust
//! Shared SDD agent runtime — consumed by score/cue/conductor frontends.
//!
//! Slice 1 surface: `Session`, `SessionEvent`, `ScoreProcess`, `ModelRouter`,
//! enough to run "create issue → author Requirements section → apply".
//! Phase advancement (`aw wi validate`) lands in slice 2.

pub mod envelope;
pub mod event;
pub mod github_backend;
pub mod gitlab_backend;
pub mod issue_backend;
pub mod jira_backend;
pub mod mainthread;
pub mod router;
pub mod score_process;
pub mod session;

pub use envelope::{parse as parse_envelope, Envelope, Invoke};
pub use event::{SessionEvent, TurnId};
pub use github_backend::GitHubIssueBackend;
pub use gitlab_backend::GitLabIssueBackend;
pub use issue_backend::{
    BackendError, BackendKind, IssueBackend, IssueBody, IssueId, IssueRef, IssueState, ListFilter,
};
pub use jira_backend::JiraIssueBackend;
pub use mainthread::{parse_decision, MainthreadDecision};
pub use router::{ModelChoice, ModelRouter, StaticRouter, Task};
pub use score_process::{
    envelope_slug, LocalIssueBackend, MockBackendCall, MockIssueBackend, MockScoreProcess,
    RealScoreProcess, ScoreCall, ScoreProcess, ScoreProcessError,
};
pub use session::{IssueBinding, Phase, Session, SessionBuilder};
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/runtime/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for the runtime module declarations and public re-export
      facade consumed by score, cue, and conductor.
```
