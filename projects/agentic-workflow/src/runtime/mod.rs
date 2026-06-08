// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/logic/runtime/mod.md#source
// CODEGEN-BEGIN
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

// CODEGEN-END
