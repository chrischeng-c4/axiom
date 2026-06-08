---
id: sdd-runtime-issue-backend
fill_sections: [overview, schema, scenarios, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue/runtime boundary logic projects AW workflow state through configured external clients."
---

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for 2 target files generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `BackendError` | projects/agentic-workflow/src/runtime/issue_backend.rs | enum | pub | 124 |  |
| `BackendKind` | projects/agentic-workflow/src/runtime/issue_backend.rs | enum | pub | 26 |  |
| `IssueBody` | projects/agentic-workflow/src/runtime/issue_backend.rs | struct | pub | 112 |  |
| `IssueId` | projects/agentic-workflow/src/runtime/issue_backend.rs | struct | pub | 52 |  |
| `IssueRef` | projects/agentic-workflow/src/runtime/issue_backend.rs | struct | pub | 99 |  |
| `IssueState` | projects/agentic-workflow/src/runtime/issue_backend.rs | enum | pub | 75 |  |
| `ListFilter` | projects/agentic-workflow/src/runtime/issue_backend.rs | struct | pub | 89 |  |
| `as_str` | projects/agentic-workflow/src/runtime/issue_backend.rs | function | pub | 36 | as_str(self) -> &'static str |
| `as_str` | projects/agentic-workflow/src/runtime/issue_backend.rs | function | pub | 60 | as_str(&self) -> &str |
| `new` | projects/agentic-workflow/src/runtime/issue_backend.rs | function | pub | 56 | new(s: impl Into<String>) -> Self |
| `LocalIssueBackend` | projects/agentic-workflow/src/runtime/score_process.rs | struct | pub | 339 |  |
| `MockBackendCall` | projects/agentic-workflow/src/runtime/score_process.rs | enum | pub | 503 |  |
| `MockIssueBackend` | projects/agentic-workflow/src/runtime/score_process.rs | struct | pub | 493 |  |
| `MockScoreProcess` | projects/agentic-workflow/src/runtime/score_process.rs | struct | pub | 211 |  |
| `RealScoreProcess` | projects/agentic-workflow/src/runtime/score_process.rs | struct | pub | 65 |  |
| `ScoreCall` | projects/agentic-workflow/src/runtime/score_process.rs | enum | pub | 171 |  |
| `ScoreProcessError` | projects/agentic-workflow/src/runtime/score_process.rs | enum | pub | 32 |  |
| `calls` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 256 | calls(&self) -> Vec<ScoreCall> |
| `calls` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 553 | calls(&self) -> Vec<MockBackendCall> |
| `enqueue_create` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 226 | enqueue_create(&self, env: Envelope) -> &Self |
| `enqueue_create` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 533 | enqueue_create(&self, id: IssueId) -> &Self |
| `enqueue_create_err` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 231 | enqueue_create_err(&self, err: ScoreProcessError) -> &Self |
| `enqueue_create_err` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 538 | enqueue_create_err(&self, err: BackendError) -> &Self |
| `enqueue_fill_section` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 236 | enqueue_fill_section(&self, env: Envelope) -> &Self |
| `enqueue_list` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 543 | enqueue_list(&self, refs: Vec<IssueRef>) -> &Self |
| `enqueue_merge` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 251 | enqueue_merge(&self, env: Envelope) -> &Self |
| `enqueue_read` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 548 | enqueue_read(&self, body: IssueBody) -> &Self |
| `enqueue_review` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 241 | enqueue_review(&self, env: Envelope) -> &Self |
| `enqueue_validate` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 246 | enqueue_validate(&self, env: Envelope) -> &Self |
| `envelope_slug` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 622 | envelope_slug(env: &Envelope) -> Option<&str> |
| `new` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 80 | new(binary: impl Into<String>) -> Self |
| `new` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 222 | new() -> Self |
| `new` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 346 | new(inner: Arc<dyn ScoreProcess>) -> Self |
| `new` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 526 | new(kind: BackendKind) -> Self |
| `verb` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 198 | verb(&self) -> &'static str |
| `with_issues_dir` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 358 | with_issues_dir(inner: Arc<dyn ScoreProcess>, issues_dir: impl Into<PathBuf>) -> Self |
| `with_project_root` | projects/agentic-workflow/src/runtime/score_process.rs | function | pub | 351 | with_project_root(inner: Arc<dyn ScoreProcess>, project_root: impl AsRef<Path>) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: sdd-runtime-issue-backend-schema

definitions:
  BackendKind:
    description: >
      Selects which IssueBackend implementation is constructed at Session
      startup. The string value matches the `[issue].backend` key in
      `.cue/config.toml`. Omitting the key defaults to `local`.
    type: string
    enum: [local, github, gitlab, jira]

  IssueId:
    description: >
      Opaque string identifier for an issue. Format is platform-specific:
      slug (local), numeric string (github/gitlab issue number), or
      project-key prefixed number (jira, e.g. "PROJ-456").
    type: string
    minLength: 1

  IssueState:
    description: Open/closed lifecycle state of an issue on any backend.
    type: string
    enum: [open, closed]

  ListFilter:
    description: >
      Filter arguments passed to IssueBackend::list. Both fields are
      optional; omitting `state` defaults to `open`.
    type: object
    properties:
      state:
        $ref: "#/definitions/IssueState"
        default: open
      labels:
        type: array
        items:
          type: string
        description: >
          Only return issues that carry ALL of the listed labels.
          Empty list means no label filter.
        default: []
    additionalProperties: false

  IssueRef:
    description: >
      Lightweight issue record used in list-view rendering. Contains only
      the fields needed to display a row in the TUI issue picker.
    type: object
    required: [id, title, state]
    properties:
      id:
        $ref: "#/definitions/IssueId"
      title:
        type: string
        minLength: 1
      state:
        $ref: "#/definitions/IssueState"
      labels:
        type: array
        items:
          type: string
        default: []
    additionalProperties: false

  IssueBody:
    description: >
      Full issue record returned by IssueBackend::read. `body_md` is the
      raw Markdown body; `frontmatter` holds parsed YAML frontmatter fields
      (local backend only — remote backends return an empty map).
    type: object
    required: [id, title, body_md]
    properties:
      id:
        $ref: "#/definitions/IssueId"
      title:
        type: string
        minLength: 1
      body_md:
        type: string
        description: Full Markdown body of the issue.
      frontmatter:
        type: object
        description: >
          YAML frontmatter fields parsed from the issue file (local backend).
          Remote backends return an empty map because platform issues do not
          carry SDD YAML frontmatter.
        additionalProperties: true
        default: {}
    additionalProperties: false

  BackendError:
    description: >
      Discriminated error type returned by IssueBackend methods. `Unsupported`
      is the canonical return for update/close on slice-1 remote backends per R8.
    type: object
    required: [kind]
    oneOf:
      - title: Unsupported
        description: >
          The operation is not supported by this backend in slice 1
          (e.g. update/close on GitHub/GitLab/Jira).
        properties:
          kind: { type: string, const: unsupported }
        required: [kind]
        additionalProperties: false

      - title: NotFound
        description: No issue with the given IssueId exists on this backend.
        properties:
          kind: { type: string, const: not_found }
          id: { $ref: "#/definitions/IssueId" }
        required: [kind, id]
        additionalProperties: false

      - title: Auth
        description: >
          Authentication failed or credentials are missing. For remote backends
          this means the required env var (GITHUB_TOKEN, GITLAB_TOKEN, or
          JIRA_API_TOKEN) is absent or rejected by the platform.
        properties:
          kind: { type: string, const: auth }
          message: { type: string }
        required: [kind, message]
        additionalProperties: false

      - title: Network
        description: >
          A subprocess invocation or HTTP request failed at the transport
          layer (non-zero exit, connection refused, timeout, etc.).
        properties:
          kind: { type: string, const: network }
          message: { type: string }
        required: [kind, message]
        additionalProperties: false

      - title: Internal
        description: Catch-all for unexpected errors within the backend impl.
        properties:
          kind: { type: string, const: internal }
          message: { type: string }
        required: [kind, message]
        additionalProperties: false

  IssueBackend:
    description: >
      Trait surface for the issue subsystem. All five methods are required
      for `local`; `update` and `close` may return BackendError::Unsupported
      for remote backends in slice 1 (R8). The trait is object-safe and
      held as `Arc<dyn IssueBackend>` inside Session.
    type: object
    properties:
      methods:
        type: array
        items:
          type: object
          required: [name, slice1_remote_allowed]
          properties:
            name:
              type: string
              enum: [create, list, read, update, close]
            slice1_remote_allowed:
              type: boolean
              description: >
                Whether this method is fully supported on remote backends in
                slice 1. False means Err(BackendError::Unsupported) is acceptable.
        default:
          - { name: create, slice1_remote_allowed: true }
          - { name: list,   slice1_remote_allowed: true }
          - { name: read,   slice1_remote_allowed: true }
          - { name: update, slice1_remote_allowed: false }
          - { name: close,  slice1_remote_allowed: false }
    additionalProperties: false
```
## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: select_backend_from_config
    title: Session construction reads config and selects the matching IssueBackend impl
    description: >
      At startup, cue reads `.cue/config.toml`. If `[issue].backend` is present
      its value selects the concrete IssueBackend implementation injected into
      Session. Omitting the key defaults to `local`.
    given:
      - A `.cue/config.toml` exists with `[issue] backend = "github"`
      - GITHUB_TOKEN env var is set to a non-empty string
    when:
      - Session is constructed via SessionBuilder
    then:
      - Session::active_backend() returns a GitHubIssueBackend instance
      - Session::active_backend().backend_kind() == BackendKind::GitHub
    acceptance:
      - test: cue::config::tests::parse_issue_backend_github
        assertion: parsed BackendKind == GitHub
      - test: cue::config::tests::parse_issue_backend_default_is_local
        assertion: omitting key returns BackendKind::Local

  - id: mainthread_routes_through_active_backend
    title: Action::SubmitChat -> Session::decide -> NewIssue dispatches to active backend
    description: >
      When the mainthread LLM classifies input as NewIssue, Session::decide
      dispatches the create call through the Arc<dyn IssueBackend> held by
      Session — not by calling ScoreProcess::create directly. This ensures
      backend selection is observed at runtime without hardcoding `local`.
    given:
      - Session is built with a MockIssueBackend configured as the active backend
      - The LLM provider mock returns '{"action":"new_issue","title":"dashboard widget"}'
        for the mainthread turn
    when:
      - Action::SubmitChat("add a dashboard widget") is dispatched
      - runner.rs delegates to Session::decide(user_input)
    then:
      - MockIssueBackend::create("dashboard widget") is called exactly once
      - ScoreProcess::create is NOT called directly (zero ScoreProcess calls recorded)
      - SessionEvent::IssueCreated { id, backend_kind: BackendKind::Local } is emitted
        when active backend is Local, or BackendKind::GitHub when it is GitHub
    acceptance:
      - test: projects/cue/tests/e2e_backend_routing.rs::newissue_routes_to_active_backend
        assertion: mock_backend.calls() has exactly one Create entry
      - test: projects/cue/tests/e2e_backend_routing.rs::local_backend_e2e_no_regression
        assertion: existing e2e tests pass with Local backend configured

  - id: local_backend_management_methods
    title: LocalIssueBackend supports list/read/close for Cue work-item management
    description: >
      Cue's work-item panes need a backend-backed management path through the
      runtime IssueBackend trait. The local runtime backend must therefore
      expose all management methods required by the trait: list, read, and close
      in addition to create/update.
    given:
      - A project has `.aw/issues/open/*.md` and `.aw/issues/closed/*.md`
        issue artifacts with SDD YAML frontmatter
      - Some open-directory issues may carry the local-only `draft` state
    when:
      - LocalIssueBackend::list is called with state open and labels
      - LocalIssueBackend::read is called with a slug
      - LocalIssueBackend::close is called with a slug
    then:
      - list returns matching open and draft issues as IssueRef records
      - read returns the Markdown body plus parsed frontmatter fields
      - close moves the issue to closed state and subsequent open lists omit it
    acceptance:
      - test: agentic_workflow::runtime::score_process::tests::local_issue_backend_lists_open_and_draft_by_label
        assertion: open list includes local draft issues and applies all label filters
      - test: agentic_workflow::runtime::score_process::tests::local_issue_backend_reads_body_and_frontmatter
        assertion: body_md and frontmatter phase/review_count are preserved
      - test: agentic_workflow::runtime::score_process::tests::local_issue_backend_closes_existing_issue
        assertion: closed issue disappears from open list and appears in closed list
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** approved

- [schema] All six required types are defined (IssueId, IssueRef, IssueBody, ListFilter, BackendError, BackendKind) plus the IssueBackend trait surface with all five methods and their slice1_remote_allowed flags. Correct and complete.
- [scenarios] Two scenarios cover config-driven backend selection and mainthread routing through the active backend, each with concrete given/when/then and named acceptance tests. Sufficient for implementation.
- [changes] Seven change entries cover the full call-path: issue_backend.rs (new trait + types), score_process.rs (LocalIssueBackend extraction), session.rs (active_backend field + routing), mod.rs (re-exports), config.rs (IssueConfig struct), tui/mod.rs (Session wiring), and e2e test file. No missing construction site. Note (secondary, non-blocking): sibling specs github_backend.md, gitlab_backend.md, jira_backend.md reference IssueId/IssueRef/IssueBody by name consistently with this spec, confirming cross-spec type alignment.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/runtime/issue_backend.rs -->
```rust
//! Issue subsystem abstraction. cue's runtime talks to issues via this
//! trait; concrete impls back it with local SDD files, GitHub Issues
//! (gh CLI), GitLab Issues (glab CLI), or Jira REST API.
//!
//! @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md
//!
//! Slice-1 contract:
//! - `create / list / read` MUST work on every backend
//! - `update / close` are required for `local`; remote backends MAY return
//!   `BackendError::Unsupported` (per issue R8 — full SDD CRRR fill
//!   semantics stay scoped to local in slice 1).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#schema
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#changes
/// Selects which backend `Session` constructs at startup. Matches the
/// `[issue].backend` key in `.cue/config.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BackendKind {
    #[default]
    Local,
    GitHub,
    GitLab,
    Jira,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl BackendKind {
    pub fn as_str(self) -> &'static str {
        match self {
            BackendKind::Local => "local",
            BackendKind::GitHub => "github",
            BackendKind::GitLab => "gitlab",
            BackendKind::Jira => "jira",
        }
    }
}

/// Opaque platform-specific issue identifier:
/// - local: slug (e.g. "add-metrics-dashboard")
/// - github / gitlab: numeric string ("123")
/// - jira: project-prefixed ("PROJ-456")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub struct IssueId(pub String);

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl IssueId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl std::fmt::Display for IssueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub enum IssueState {
    Open,
    Closed,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl Default for IssueState {
    fn default() -> Self {
        IssueState::Open
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub struct ListFilter {
    #[serde(default)]
    pub state: IssueState,
    #[serde(default)]
    pub labels: Vec<String>,
}

/// List-view record — minimum fields for the TUI issue picker row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub struct IssueRef {
    pub id: IssueId,
    pub title: String,
    pub state: IssueState,
    #[serde(default)]
    pub labels: Vec<String>,
}

/// Full issue record returned by `read`. `frontmatter` is populated only
/// for the local backend (remote platforms have no SDD YAML frontmatter
/// concept; their backends return an empty map).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub struct IssueBody {
    pub id: IssueId,
    pub title: String,
    pub body_md: String,
    #[serde(default)]
    pub frontmatter: BTreeMap<String, serde_json::Value>,
}

/// Discriminated error type. `Unsupported` is the canonical return for
/// `update` / `close` on slice-1 remote backends (R8).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub enum BackendError {
    #[error("operation not supported by this backend in slice 1")]
    Unsupported,

    #[error("issue not found: {0}")]
    NotFound(IssueId),

    #[error("auth failed: {0}")]
    Auth(String),

    #[error("network/transport error: {0}")]
    Network(String),

    #[error("internal backend error: {0}")]
    Internal(String),
}

/// Issue subsystem trait — the abstraction `Session` holds via
/// `Arc<dyn IssueBackend>`. Object-safe.
#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub trait IssueBackend: Send + Sync {
    /// Identifies which concrete impl this is (for the TUI status bar
    /// and SessionEvent payloads).
    fn backend_kind(&self) -> BackendKind;

    /// Create a new issue with the given title. Returns the platform's
    /// canonical issue id.
    async fn create(&self, title: &str) -> Result<IssueId, BackendError>;

    /// List issues matching `filter`. The `state` field defaults to
    /// `Open`; `labels` empty means no label filter.
    async fn list(&self, filter: &ListFilter) -> Result<Vec<IssueRef>, BackendError>;

    /// Read the full issue body for a given id.
    async fn read(&self, id: &IssueId) -> Result<IssueBody, BackendError>;

    /// Update a section of the issue body. Slice 1: required on local;
    /// remote backends MAY return `Unsupported`.
    async fn update(&self, id: &IssueId, section: &str, body: &str) -> Result<(), BackendError>;

    /// Close the issue with an optional message. Slice 1: required on
    /// local; remote backends MAY return `Unsupported`.
    async fn close(&self, id: &IssueId, message: Option<&str>) -> Result<(), BackendError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_kind_serde_roundtrip_lowercase() {
        let k = BackendKind::GitHub;
        let s = serde_json::to_string(&k).unwrap();
        assert_eq!(s, "\"github\"");
        let back: BackendKind = serde_json::from_str(&s).unwrap();
        assert_eq!(back, BackendKind::GitHub);
    }

    #[test]
    fn backend_kind_default_is_local() {
        assert_eq!(BackendKind::default(), BackendKind::Local);
    }

    #[test]
    fn list_filter_default_state_is_open() {
        let f = ListFilter::default();
        assert_eq!(f.state, IssueState::Open);
        assert!(f.labels.is_empty());
    }

    #[test]
    fn issue_id_display_round_trip() {
        let id = IssueId::new("add-metrics-dashboard");
        assert_eq!(format!("{id}"), "add-metrics-dashboard");
    }

    #[test]
    fn backend_error_display() {
        let e = BackendError::Unsupported;
        assert!(e.to_string().contains("not supported"));
        let nf = BackendError::NotFound(IssueId::new("abc"));
        assert!(nf.to_string().contains("abc"));
    }
}
```

<!-- source-snapshot: path=projects/agentic-workflow/src/runtime/score_process.rs -->
```rust
//! Trait + impls for invoking the `score` CLI from a Rust process.
//!
//! `aw wi` and `aw td` are CLI-only today (no Rust library API),
//! so the real impl shells out via `tokio::process::Command` and parses the
//! envelope JSON from stdout. A `MockScoreProcess` lets tests run without a real
//! `score` binary.
//!
//! The trait takes `serde_json::Value` for fill-section args so the schema
//! stays in lockstep with the CLI's `Invoke.args` shape.

use crate::issues::{
    Issue as StoredIssue, IssueBackend as StoredIssueBackend, IssueFilter as StoredIssueFilter,
    IssueState as StoredIssueState, LocalBackend,
};
use crate::runtime::envelope::Envelope;
use crate::runtime::issue_backend::{
    BackendError, BackendKind, IssueBackend, IssueBody, IssueId, IssueRef,
    IssueState as RuntimeIssueState, ListFilter,
};
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::process::Command;

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#changes
#[derive(Debug, Error)]
pub enum ScoreProcessError {
    #[error("aw binary not found on PATH")]
    BinaryNotFound,
    #[error("score exited non-zero ({code:?}): {stderr}")]
    NonZeroExit { code: Option<i32>, stderr: String },
    #[error("could not parse envelope JSON: {source}\n--- stdout ---\n{stdout}")]
    ParseEnvelope {
        source: serde_json::Error,
        stdout: String,
    },
    #[error("io error invoking score: {0}")]
    Io(#[from] std::io::Error),
    #[error("mock has no canned response queued for `{0}`")]
    MockExhausted(String),
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub trait ScoreProcess: Send + Sync {
    async fn create(&self, title: &str) -> Result<Envelope, ScoreProcessError>;
    async fn fill_section_apply(
        &self,
        slug: &str,
        section: &str,
        body: &str,
    ) -> Result<Envelope, ScoreProcessError>;
    async fn review_apply(&self, slug: &str, body: &str) -> Result<Envelope, ScoreProcessError>;
    async fn validate(&self, slug: &str) -> Result<Envelope, ScoreProcessError>;
    async fn merge(&self, slug: &str) -> Result<Envelope, ScoreProcessError>;
}

/// Real impl — shells out to the `score` binary on PATH.
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub struct RealScoreProcess {
    pub binary: String,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl Default for RealScoreProcess {
    fn default() -> Self {
        Self {
            binary: "aw".to_string(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl RealScoreProcess {
    pub fn new(binary: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
        }
    }

    async fn run(&self, args: &[&str]) -> Result<Envelope, ScoreProcessError> {
        let output = Command::new(&self.binary)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            return Err(ScoreProcessError::NonZeroExit {
                code: output.status.code(),
                stderr,
            });
        }

        // Score may print prelude lines before the JSON envelope on the last
        // line. Strategy: try the whole stdout, then walk back from the end
        // looking for a parseable JSON object.
        if let Ok(env) = serde_json::from_str::<Envelope>(stdout.trim()) {
            return Ok(env);
        }
        for line in stdout.lines().rev() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(env) = serde_json::from_str::<Envelope>(trimmed) {
                return Ok(env);
            }
        }
        Err(ScoreProcessError::ParseEnvelope {
            source: serde_json::from_str::<Envelope>(stdout.trim()).unwrap_err(),
            stdout,
        })
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl ScoreProcess for RealScoreProcess {
    async fn create(&self, title: &str) -> Result<Envelope, ScoreProcessError> {
        self.run(&["issues", "create", title]).await
    }

    async fn fill_section_apply(
        &self,
        slug: &str,
        section: &str,
        body: &str,
    ) -> Result<Envelope, ScoreProcessError> {
        self.run(&[
            "issues",
            "fill-section",
            "--apply",
            "--slug",
            slug,
            "--section",
            section,
            "--body",
            body,
        ])
        .await
    }

    async fn validate(&self, slug: &str) -> Result<Envelope, ScoreProcessError> {
        self.run(&["issues", "validate", slug]).await
    }

    async fn review_apply(&self, slug: &str, body: &str) -> Result<Envelope, ScoreProcessError> {
        self.run(&[
            "issues", "review", "--apply", "--slug", slug, "--body", body,
        ])
        .await
    }

    async fn merge(&self, slug: &str) -> Result<Envelope, ScoreProcessError> {
        self.run(&["issues", "merge", slug]).await
    }
}

/// Mock impl — feed it canned envelopes and assert on recorded calls.
#[derive(Debug, Clone, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub enum ScoreCall {
    Create {
        title: String,
    },
    FillSectionApply {
        slug: String,
        section: String,
        body: String,
    },
    ReviewApply {
        slug: String,
        body: String,
    },
    Validate {
        slug: String,
    },
    Merge {
        slug: String,
    },
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl ScoreCall {
    /// Stable verb token matching the CLI subcommand path. Lifecycle harness
    /// uses this to wait for "the next time MockScoreProcess sees `validate`".
    /// Tokens: `"create"`, `"fill_section"`, `"review_apply"`, `"validate"`,
    /// `"merge"`.
    pub fn verb(&self) -> &'static str {
        match self {
            ScoreCall::Create { .. } => "create",
            ScoreCall::FillSectionApply { .. } => "fill_section",
            ScoreCall::ReviewApply { .. } => "review_apply",
            ScoreCall::Validate { .. } => "validate",
            ScoreCall::Merge { .. } => "merge",
        }
    }
}

#[derive(Default)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub struct MockScoreProcess {
    create_responses: Mutex<Vec<Result<Envelope, ScoreProcessError>>>,
    fill_responses: Mutex<Vec<Result<Envelope, ScoreProcessError>>>,
    review_responses: Mutex<Vec<Result<Envelope, ScoreProcessError>>>,
    validate_responses: Mutex<Vec<Result<Envelope, ScoreProcessError>>>,
    merge_responses: Mutex<Vec<Result<Envelope, ScoreProcessError>>>,
    calls: Mutex<Vec<ScoreCall>>,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl MockScoreProcess {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue_create(&self, env: Envelope) -> &Self {
        self.create_responses.lock().unwrap().push(Ok(env));
        self
    }

    pub fn enqueue_create_err(&self, err: ScoreProcessError) -> &Self {
        self.create_responses.lock().unwrap().push(Err(err));
        self
    }

    pub fn enqueue_fill_section(&self, env: Envelope) -> &Self {
        self.fill_responses.lock().unwrap().push(Ok(env));
        self
    }

    pub fn enqueue_review(&self, env: Envelope) -> &Self {
        self.review_responses.lock().unwrap().push(Ok(env));
        self
    }

    pub fn enqueue_validate(&self, env: Envelope) -> &Self {
        self.validate_responses.lock().unwrap().push(Ok(env));
        self
    }

    pub fn enqueue_merge(&self, env: Envelope) -> &Self {
        self.merge_responses.lock().unwrap().push(Ok(env));
        self
    }

    pub fn calls(&self) -> Vec<ScoreCall> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl ScoreProcess for MockScoreProcess {
    async fn create(&self, title: &str) -> Result<Envelope, ScoreProcessError> {
        self.calls.lock().unwrap().push(ScoreCall::Create {
            title: title.to_string(),
        });
        let mut q = self.create_responses.lock().unwrap();
        if q.is_empty() {
            return Err(ScoreProcessError::MockExhausted("create".into()));
        }
        q.remove(0)
    }

    async fn fill_section_apply(
        &self,
        slug: &str,
        section: &str,
        body: &str,
    ) -> Result<Envelope, ScoreProcessError> {
        self.calls
            .lock()
            .unwrap()
            .push(ScoreCall::FillSectionApply {
                slug: slug.to_string(),
                section: section.to_string(),
                body: body.to_string(),
            });
        let mut q = self.fill_responses.lock().unwrap();
        if q.is_empty() {
            return Err(ScoreProcessError::MockExhausted(
                "fill_section_apply".into(),
            ));
        }
        q.remove(0)
    }

    async fn validate(&self, slug: &str) -> Result<Envelope, ScoreProcessError> {
        self.calls.lock().unwrap().push(ScoreCall::Validate {
            slug: slug.to_string(),
        });
        let mut q = self.validate_responses.lock().unwrap();
        if q.is_empty() {
            return Err(ScoreProcessError::MockExhausted("validate".into()));
        }
        q.remove(0)
    }

    async fn review_apply(&self, slug: &str, body: &str) -> Result<Envelope, ScoreProcessError> {
        self.calls.lock().unwrap().push(ScoreCall::ReviewApply {
            slug: slug.to_string(),
            body: body.to_string(),
        });
        let mut q = self.review_responses.lock().unwrap();
        if q.is_empty() {
            return Err(ScoreProcessError::MockExhausted("review_apply".into()));
        }
        q.remove(0)
    }

    async fn merge(&self, slug: &str) -> Result<Envelope, ScoreProcessError> {
        self.calls.lock().unwrap().push(ScoreCall::Merge {
            slug: slug.to_string(),
        });
        let mut q = self.merge_responses.lock().unwrap();
        if q.is_empty() {
            return Err(ScoreProcessError::MockExhausted("merge".into()));
        }
        q.remove(0)
    }
}

// ── IssueBackend impls ──────────────────────────────────────────────

/// Local SDD-file backend. Wraps an inner `Arc<dyn ScoreProcess>` for lifecycle
/// operations while using the local issue store for read/list/close.
///
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#changes
pub struct LocalIssueBackend {
    inner: Arc<dyn ScoreProcess>,
    issues_dir: PathBuf,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl LocalIssueBackend {
    pub fn new(inner: Arc<dyn ScoreProcess>) -> Self {
        let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::with_project_root(inner, project_root)
    }

    pub fn with_project_root(inner: Arc<dyn ScoreProcess>, project_root: impl AsRef<Path>) -> Self {
        Self::with_issues_dir(
            inner,
            crate::shared::workspace::issues_path(project_root.as_ref()),
        )
    }

    pub fn with_issues_dir(inner: Arc<dyn ScoreProcess>, issues_dir: impl Into<PathBuf>) -> Self {
        Self {
            inner,
            issues_dir: issues_dir.into(),
        }
    }

    fn local_store(&self) -> LocalBackend {
        LocalBackend::at(self.issues_dir.clone())
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl IssueBackend for LocalIssueBackend {
    fn backend_kind(&self) -> BackendKind {
        BackendKind::Local
    }

    async fn create(&self, title: &str) -> Result<IssueId, BackendError> {
        let env = self
            .inner
            .create(title)
            .await
            .map_err(|e| BackendError::Internal(format!("score process: {e}")))?;
        let slug = envelope_slug(&env).ok_or_else(|| {
            BackendError::Internal("score create returned envelope with no slug".into())
        })?;
        Ok(IssueId::new(slug))
    }

    async fn list(&self, filter: &ListFilter) -> Result<Vec<IssueRef>, BackendError> {
        let store = self.local_store();
        let issues = StoredIssueBackend::list(&store, &StoredIssueFilter::default())
            .await
            .map_err(local_store_error)?;

        Ok(issues
            .into_iter()
            .filter(|issue| runtime_filter_matches(issue, filter))
            .map(issue_ref_from_stored)
            .collect())
    }

    async fn read(&self, id: &IssueId) -> Result<IssueBody, BackendError> {
        let store = self.local_store();
        let issue = StoredIssueBackend::get(&store, id.as_str())
            .await
            .map_err(local_store_error)?
            .ok_or_else(|| BackendError::NotFound(id.clone()))?;
        issue_body_from_stored(&issue)
    }

    async fn update(&self, id: &IssueId, section: &str, body: &str) -> Result<(), BackendError> {
        self.inner
            .fill_section_apply(id.as_str(), section, body)
            .await
            .map(|_| ())
            .map_err(|e| BackendError::Internal(format!("fill_section_apply: {e}")))
    }

    async fn close(&self, id: &IssueId, message: Option<&str>) -> Result<(), BackendError> {
        let store = self.local_store();
        if StoredIssueBackend::get(&store, id.as_str())
            .await
            .map_err(local_store_error)?
            .is_none()
        {
            return Err(BackendError::NotFound(id.clone()));
        }

        StoredIssueBackend::close(&store, id.as_str(), message)
            .await
            .map_err(local_store_error)
    }
}

fn runtime_filter_matches(issue: &StoredIssue, filter: &ListFilter) -> bool {
    if stored_state_to_runtime(issue.state) != filter.state {
        return false;
    }

    filter
        .labels
        .iter()
        .all(|needle| issue.labels.iter().any(|label| label == needle))
}

fn stored_state_to_runtime(state: StoredIssueState) -> RuntimeIssueState {
    match state {
        StoredIssueState::Closed => RuntimeIssueState::Closed,
        StoredIssueState::Open | StoredIssueState::Draft => RuntimeIssueState::Open,
    }
}

fn issue_ref_from_stored(issue: StoredIssue) -> IssueRef {
    IssueRef {
        id: IssueId::new(issue.slug),
        title: issue.title,
        state: stored_state_to_runtime(issue.state),
        labels: issue.labels,
    }
}

fn issue_body_from_stored(issue: &StoredIssue) -> Result<IssueBody, BackendError> {
    Ok(IssueBody {
        id: IssueId::new(issue.slug.clone()),
        title: issue.title.clone(),
        body_md: issue.body.clone(),
        frontmatter: frontmatter_from_stored(issue)?,
    })
}

fn frontmatter_from_stored(
    issue: &StoredIssue,
) -> Result<BTreeMap<String, serde_json::Value>, BackendError> {
    let value = serde_json::to_value(issue)
        .map_err(|e| BackendError::Internal(format!("local issue frontmatter: {e}")))?;
    match value {
        serde_json::Value::Object(map) => Ok(map.into_iter().collect()),
        _ => Err(BackendError::Internal(
            "local issue did not serialize to an object".into(),
        )),
    }
}

fn local_store_error(error: anyhow::Error) -> BackendError {
    BackendError::Internal(format!("local issue store: {error}"))
}

/// Mock backend for tests — records every call, returns canned results
/// from queues. Default behavior: `create` pops from a queue of
/// canned ids; if the queue is empty, returns `BackendError::Internal`.
#[derive(Default)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub struct MockIssueBackend {
    kind: BackendKind,
    create_responses: Mutex<Vec<Result<IssueId, BackendError>>>,
    list_responses: Mutex<Vec<Result<Vec<IssueRef>, BackendError>>>,
    read_responses: Mutex<Vec<Result<IssueBody, BackendError>>>,
    calls: Mutex<Vec<MockBackendCall>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub enum MockBackendCall {
    Create {
        title: String,
    },
    List {
        filter: ListFilter,
    },
    Read {
        id: IssueId,
    },
    Update {
        id: IssueId,
        section: String,
        body: String,
    },
    Close {
        id: IssueId,
        message: Option<String>,
    },
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl MockIssueBackend {
    pub fn new(kind: BackendKind) -> Self {
        Self {
            kind,
            ..Self::default()
        }
    }

    pub fn enqueue_create(&self, id: IssueId) -> &Self {
        self.create_responses.lock().unwrap().push(Ok(id));
        self
    }

    pub fn enqueue_create_err(&self, err: BackendError) -> &Self {
        self.create_responses.lock().unwrap().push(Err(err));
        self
    }

    pub fn enqueue_list(&self, refs: Vec<IssueRef>) -> &Self {
        self.list_responses.lock().unwrap().push(Ok(refs));
        self
    }

    pub fn enqueue_read(&self, body: IssueBody) -> &Self {
        self.read_responses.lock().unwrap().push(Ok(body));
        self
    }

    pub fn calls(&self) -> Vec<MockBackendCall> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl IssueBackend for MockIssueBackend {
    fn backend_kind(&self) -> BackendKind {
        self.kind
    }

    async fn create(&self, title: &str) -> Result<IssueId, BackendError> {
        self.calls.lock().unwrap().push(MockBackendCall::Create {
            title: title.to_string(),
        });
        let mut q = self.create_responses.lock().unwrap();
        if q.is_empty() {
            return Err(BackendError::Internal(
                "MockIssueBackend: no canned create".into(),
            ));
        }
        q.remove(0)
    }

    async fn list(&self, filter: &ListFilter) -> Result<Vec<IssueRef>, BackendError> {
        self.calls.lock().unwrap().push(MockBackendCall::List {
            filter: filter.clone(),
        });
        let mut q = self.list_responses.lock().unwrap();
        if q.is_empty() {
            return Ok(vec![]);
        }
        q.remove(0)
    }

    async fn read(&self, id: &IssueId) -> Result<IssueBody, BackendError> {
        self.calls
            .lock()
            .unwrap()
            .push(MockBackendCall::Read { id: id.clone() });
        let mut q = self.read_responses.lock().unwrap();
        if q.is_empty() {
            return Err(BackendError::NotFound(id.clone()));
        }
        q.remove(0)
    }

    async fn update(&self, id: &IssueId, section: &str, body: &str) -> Result<(), BackendError> {
        self.calls.lock().unwrap().push(MockBackendCall::Update {
            id: id.clone(),
            section: section.to_string(),
            body: body.to_string(),
        });
        Ok(())
    }

    async fn close(&self, id: &IssueId, message: Option<&str>) -> Result<(), BackendError> {
        self.calls.lock().unwrap().push(MockBackendCall::Close {
            id: id.clone(),
            message: message.map(|s| s.to_string()),
        });
        Ok(())
    }
}

/// Sanity helper used by router/session impls: pull the slug out of a
/// `Dispatch`/`Done`/`Error` envelope. Returns None for `Batch`.
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub fn envelope_slug(env: &Envelope) -> Option<&str> {
    match env {
        Envelope::Dispatch { slug, .. }
        | Envelope::Done { slug, .. }
        | Envelope::Error { slug, .. } => Some(slug.as_str()),
        Envelope::Batch { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn done(slug: &str) -> Envelope {
        Envelope::Done {
            slug: slug.into(),
            message: None,
        }
    }

    #[tokio::test]
    async fn mock_records_create_call_and_returns_canned() {
        let mock = MockScoreProcess::new();
        mock.enqueue_create(done("abc"));
        let env = mock.create("hello world").await.unwrap();
        assert_eq!(envelope_slug(&env), Some("abc"));
        assert_eq!(
            mock.calls(),
            vec![ScoreCall::Create {
                title: "hello world".into()
            }]
        );
    }

    #[tokio::test]
    async fn mock_records_fill_section_call() {
        let mock = MockScoreProcess::new();
        mock.enqueue_fill_section(done("abc"));
        mock.fill_section_apply("abc", "requirements", "## body")
            .await
            .unwrap();
        assert_eq!(
            mock.calls(),
            vec![ScoreCall::FillSectionApply {
                slug: "abc".into(),
                section: "requirements".into(),
                body: "## body".into(),
            }]
        );
    }

    #[tokio::test]
    async fn mock_exhausted_returns_err() {
        let mock = MockScoreProcess::new();
        let err = mock.create("x").await.unwrap_err();
        assert!(matches!(err, ScoreProcessError::MockExhausted(_)));
    }

    fn stored_issue(slug: &str, state: StoredIssueState, labels: Vec<&str>) -> StoredIssue {
        StoredIssue {
            issue_type: crate::issues::IssueType::Enhancement,
            title: format!("test {slug}"),
            state,
            id: None,
            github_id: None,
            gitlab_id: None,
            url: None,
            author: Some("tester".into()),
            labels: labels.into_iter().map(str::to_string).collect(),
            created_at: None,
            updated_at: None,
            slug: slug.to_string(),
            body: "## Problem\n\nBody content.".into(),
            related: vec![],
            implements: vec![],
            phase: None,
            branch: None,
            target_branch: None,
            git_workflow: None,
            change_id: None,
            iteration: None,
            current_task_id: None,
            impl_spec_phase: None,
            task_revisions: None,
            revision_counts: None,
            last_action: None,
            session_id: None,
            validation_errors: vec![],
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        }
    }

    async fn write_stored_issue(issues_dir: &std::path::Path, issue: &StoredIssue) {
        let store = LocalBackend::at(issues_dir.to_path_buf());
        StoredIssueBackend::write(&store, issue).await.unwrap();
    }

    fn local_issue_backend(issues_dir: &std::path::Path) -> LocalIssueBackend {
        LocalIssueBackend::with_issues_dir(
            Arc::new(MockScoreProcess::new()),
            issues_dir.to_path_buf(),
        )
    }

    #[tokio::test]
    async fn local_issue_backend_lists_open_and_draft_by_label() {
        let tmp = tempfile::TempDir::new().unwrap();
        write_stored_issue(
            tmp.path(),
            &stored_issue(
                "open-cue",
                StoredIssueState::Open,
                vec!["project:cue", "priority:p1"],
            ),
        )
        .await;
        write_stored_issue(
            tmp.path(),
            &stored_issue(
                "draft-cue",
                StoredIssueState::Draft,
                vec!["project:cue", "priority:p1"],
            ),
        )
        .await;
        write_stored_issue(
            tmp.path(),
            &stored_issue(
                "closed-cue",
                StoredIssueState::Closed,
                vec!["project:cue", "priority:p1"],
            ),
        )
        .await;
        write_stored_issue(
            tmp.path(),
            &stored_issue("open-other", StoredIssueState::Open, vec!["project:cue"]),
        )
        .await;

        let backend = local_issue_backend(tmp.path());
        let refs = backend
            .list(&ListFilter {
                state: RuntimeIssueState::Open,
                labels: vec!["project:cue".into(), "priority:p1".into()],
            })
            .await
            .unwrap();
        let ids: Vec<_> = refs.into_iter().map(|r| r.id.0).collect();

        assert_eq!(ids, vec!["draft-cue", "open-cue"]);
    }

    #[tokio::test]
    async fn local_issue_backend_reads_body_and_frontmatter() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut issue = stored_issue(
            "enhancement-read",
            StoredIssueState::Open,
            vec!["project:cue"],
        );
        issue.phase = Some("reviewed".into());
        issue.review_count = Some(1);
        write_stored_issue(tmp.path(), &issue).await;

        let backend = local_issue_backend(tmp.path());
        let body = backend
            .read(&IssueId::new("enhancement-read"))
            .await
            .unwrap();

        assert_eq!(body.id.as_str(), "enhancement-read");
        assert!(body.body_md.contains("Body content"));
        assert_eq!(
            body.frontmatter.get("phase").and_then(|v| v.as_str()),
            Some("reviewed")
        );
        assert_eq!(
            body.frontmatter
                .get("review_count")
                .and_then(|v| v.as_u64()),
            Some(1)
        );
    }

    #[tokio::test]
    async fn local_issue_backend_read_missing_returns_not_found() {
        let tmp = tempfile::TempDir::new().unwrap();
        let backend = local_issue_backend(tmp.path());

        let err = backend.read(&IssueId::new("missing")).await.unwrap_err();

        assert!(matches!(err, BackendError::NotFound(id) if id.as_str() == "missing"));
    }

    #[tokio::test]
    async fn local_issue_backend_closes_existing_issue() {
        let tmp = tempfile::TempDir::new().unwrap();
        write_stored_issue(
            tmp.path(),
            &stored_issue("enhancement-close", StoredIssueState::Open, vec![]),
        )
        .await;

        let backend = local_issue_backend(tmp.path());
        backend
            .close(&IssueId::new("enhancement-close"), Some("done"))
            .await
            .unwrap();

        let open = backend.list(&ListFilter::default()).await.unwrap();
        assert!(open.is_empty());

        let closed = backend
            .list(&ListFilter {
                state: RuntimeIssueState::Closed,
                labels: vec![],
            })
            .await
            .unwrap();
        assert_eq!(closed.len(), 1);
        assert_eq!(closed[0].id.as_str(), "enhancement-close");
        assert_eq!(closed[0].state, RuntimeIssueState::Closed);
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
# Runtime issue backend contract and score process bridge are fully
# regenerable through the source template. The remaining sibling entries stay
# hand-written because they belong to separate runtime, config, and TUI
# ownership boundaries.
changes:
  - path: projects/agentic-workflow/src/runtime/issue_backend.rs
    action: modify
    section: source
    impl_mode: codegen
    description: >
      Source template for the IssueBackend async trait and runtime DTO/error
      contract: BackendKind, IssueId, IssueState, IssueRef, IssueBody,
      ListFilter, and BackendError.

  - path: projects/agentic-workflow/src/runtime/score_process.rs
    action: modify
    section: source
    impl_mode: codegen
    description: >
      Source template for the ScoreProcess trait, RealScoreProcess CLI adapter,
      MockScoreProcess test double, LocalIssueBackend bridge, MockIssueBackend,
      and envelope_slug helper. The template preserves the async subprocess and
      local store behavior while making this runtime integration layer fully
      regenerable.

  - path: projects/agentic-workflow/src/runtime/session.rs
    action: modify
    section: source
    impl_mode: hand-written
    description: >
      Replaces the direct Arc<dyn ScoreProcess> create call in run_create_issue
      with a call through Arc<dyn IssueBackend>. Session gains a second
      Arc<dyn IssueBackend> field (active_backend) alongside score_process.
      Session::decide dispatches NewIssue through active_backend.create
      rather than calling score_process.create directly. SessionBuilder gains
      a corresponding issue_backend() setter.

  - path: projects/agentic-workflow/src/runtime/mod.rs
    action: modify
    section: source
    impl_mode: hand-written
    description: >
      Adds pub mod issue_backend and re-exports IssueBackend, IssueId,
      IssueRef, IssueBody, ListFilter, BackendError, and BackendKind so
      callers can import them via agentic_workflow::runtime::{IssueBackend, BackendKind, ...}.

  - path: projects/cue/src/config.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: >
      Adds IssueConfig struct { backend: BackendKind } with serde default
      of BackendKind::Local. CueConfig gains an `issue: IssueConfig` field.
      Parsing from `.cue/config.toml` handles missing `[issue]` table by
      falling back to IssueConfig::default().

  - path: projects/cue/src/tui/mod.rs
    action: modify
    section: source
    impl_mode: hand-written
    description: >
      Session construction reads the parsed IssueConfig::backend from CueConfig
      and constructs the matching IssueBackend impl (LocalIssueBackend for Local,
      GitHubIssueBackend for GitHub, GitLabIssueBackend for GitLab,
      JiraIssueBackend for Jira). The concrete backend is wrapped in Arc and
      passed to SessionBuilder::issue_backend().

  - path: projects/cue/tests/e2e_backend_routing.rs
    action: create
    section: source
    impl_mode: hand-written
    description: >
      New test file. Adds newissue_routes_to_active_backend: mocks IssueBackend,
      configures Session with it, dispatches Action::SubmitChat with a NewIssue-
      classified LLM response, and asserts mock_backend.calls() has exactly one
      Create entry. Adds local_backend_e2e_no_regression: runs the full lifecycle
      with LocalIssueBackend and asserts existing assertions from e2e_lifecycle.rs
      still hold.
  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

```

## Traceability Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."
```
