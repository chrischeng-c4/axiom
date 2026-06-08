---
id: sdd-runtime-github-backend
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

Public API manifest for `projects/agentic-workflow/src/runtime/github_backend.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GitHubIssueBackend` | projects/agentic-workflow/src/runtime/github_backend.rs | struct | pub | 28 |  |
| `from_env` | projects/agentic-workflow/src/runtime/github_backend.rs | function | pub | 37 | from_env(repo: Option<String>) -> Result<Self, BackendError> |
| `with_binary` | projects/agentic-workflow/src/runtime/github_backend.rs | function | pub | 48 | with_binary(mut self, binary: impl Into<String>) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: sdd-runtime-github-backend-schema

definitions:
  GitHubIssueNumber:
    description: >
      GitHub issue number as returned by the gh CLI — a positive integer
      serialised as a string to match the IssueId contract.
    type: string
    pattern: "^[1-9][0-9]*$"

  GhIssueCreateArgs:
    description: >
      Arguments passed to `gh issue create` subprocess. `repo` is optional;
      when absent gh infers the repository from the current git remote.
      `body` carries the issue description markdown.
    type: object
    required: [title]
    properties:
      repo:
        type: string
        description: >
          `--repo OWNER/REPO` passed to gh. Sourced from `[issue].github_repo`
          in `.cue/config.toml` when present; otherwise omitted.
      title:
        type: string
        minLength: 1
      body:
        type: string
        default: ""
    additionalProperties: false

  GhIssueJsonFields:
    description: >
      Subset of fields that `gh issue list --json` and `gh issue view --json`
      return. Only the fields required by IssueRef and IssueBody are listed.
    type: object
    required: [number, title, state]
    properties:
      number:
        type: integer
        description: GitHub issue number (used as IssueId, stringified).
      title:
        type: string
      state:
        type: string
        enum: [OPEN, CLOSED]
        description: >
          GitHub API state is uppercase; GitHubIssueBackend lowercases it
          when constructing IssueRef / IssueBody so it matches IssueState.
      labels:
        type: array
        items:
          type: object
          required: [name]
          properties:
            name: { type: string }
          additionalProperties: true
        default: []
      body:
        type: string
        description: Full issue body markdown. Present in `view` output only.
    additionalProperties: true

  GhListArgs:
    description: >
      Arguments for `gh issue list --json number,title,state,labels`.
      `state` maps from ListFilter.state: open -> open, closed -> closed,
      all -> all (gh CLI values).
    type: object
    required: [json_fields]
    properties:
      state:
        type: string
        enum: [open, closed, all]
        default: open
      labels:
        type: array
        items: { type: string }
        description: >
          Each label passed as a separate `--label` flag. Empty list omits
          the flag entirely (no label filtering).
        default: []
      json_fields:
        type: string
        const: "number,title,state,labels"
        description: Fixed JSON field list passed to --json flag.
    additionalProperties: false

  GhAuthConfig:
    description: >
      Authentication configuration for GitHubIssueBackend. Credentials are
      read-only at construction; no credential storage is performed (R10).
      The gh CLI reads GITHUB_TOKEN from the environment natively.
    type: object
    properties:
      token_env_var:
        type: string
        const: "GITHUB_TOKEN"
        description: >
          Name of the environment variable holding the GitHub personal access
          token. The gh CLI reads this natively; GitHubIssueBackend checks for
          its presence at construction and returns BackendError::Auth if absent.
      repo:
        type: string
        description: >
          Optional `OWNER/REPO` override from `[issue].github_repo` in
          `.cue/config.toml`. When absent gh infers from the git remote.
    additionalProperties: false
```
## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: github_create_happy_path
    title: GitHubIssueBackend creates an issue via gh CLI subprocess and returns IssueId
    description: >
      When create(title) is called and a mock gh subprocess returns a
      valid JSON object with `number`, the backend extracts the number,
      stringifies it, and returns it as IssueId. No network call is made —
      the test replaces the gh binary path with a mock executable.
    given:
      - GITHUB_TOKEN env var is set to "test-token"
      - The mock gh binary is configured to print
        '{"number": 42, "title": "new widget", "state": "OPEN", "labels": []}'
        on stdout with exit code 0 when given `issue create` args
    when:
      - GitHubIssueBackend::create("new widget") is called
    then:
      - The returned IssueId equals "42"
      - The subprocess was called with args including `issue create --title "new widget"`
    acceptance:
      - test: projects/agentic-workflow/tests/github_backend_tests.rs::create_happy_path
        assertion: result == Ok(IssueId::from("42"))

  - id: github_auth_missing
    title: GitHubIssueBackend returns BackendError::Auth when GITHUB_TOKEN is absent
    description: >
      GitHubIssueBackend checks for the GITHUB_TOKEN env var at construction
      time. When the var is absent, all five IssueBackend methods return
      Err(BackendError::Auth) without invoking the gh subprocess.
    given:
      - GITHUB_TOKEN env var is NOT set in the test environment
    when:
      - GitHubIssueBackend is constructed and create("title") is called
    then:
      - create returns Err(BackendError::Auth { message: contains "GITHUB_TOKEN" })
      - No subprocess was spawned
    acceptance:
      - test: projects/agentic-workflow/tests/github_backend_tests.rs::auth_missing_returns_error
        assertion: matches!(err, BackendError::Auth { .. })
```
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/runtime/github_backend.rs -->
```rust
//! GitHub Issues backend — shells out to the `gh` CLI.
//!
//! @spec projects/agentic-workflow/tech-design/core/logic/runtime/github_backend.md
//!
//! Slice 1: create / list / read (R2). update / close return
//! `BackendError::Unsupported` (R8 — SDD CRRR fill semantics stay
//! local in slice 1).
//!
//! Auth: `GITHUB_TOKEN` env var, read natively by the `gh` CLI.
//! Backend checks for its presence at construction; absent → all
//! methods return `BackendError::Auth`.

use crate::runtime::issue_backend::{
    BackendError, BackendKind, IssueBackend, IssueBody, IssueId, IssueRef, IssueState, ListFilter,
};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::process::Stdio;
use tokio::process::Command;

const TOKEN_ENV_VAR: &str = "GITHUB_TOKEN";

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/github_backend.md#changes
#[derive(Debug)]
pub struct GitHubIssueBackend {
    binary: String,
    repo: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/github_backend.md#source
impl GitHubIssueBackend {
    /// Construct from environment. Returns `BackendError::Auth` if
    /// `GITHUB_TOKEN` is not set.
    pub fn from_env(repo: Option<String>) -> Result<Self, BackendError> {
        let _ = std::env::var(TOKEN_ENV_VAR)
            .map_err(|_| BackendError::Auth(format!("{TOKEN_ENV_VAR} env var is not set")))?;
        Ok(Self {
            binary: "gh".into(),
            repo,
        })
    }

    /// Test-only override: point the backend at a custom gh binary
    /// (typically a shell script in a tempdir that mocks gh output).
    pub fn with_binary(mut self, binary: impl Into<String>) -> Self {
        self.binary = binary.into();
        self
    }
}

#[derive(Debug, Deserialize)]
struct GhIssueJson {
    number: u64,
    title: String,
    state: String,
    #[serde(default)]
    labels: Vec<GhLabel>,
    #[serde(default)]
    body: String,
}

#[derive(Debug, Deserialize)]
struct GhLabel {
    name: String,
}

fn parse_state(s: &str) -> IssueState {
    // gh `--json` output gives uppercase (OPEN/CLOSED) via GraphQL;
    // gh's REST-backed paths give lowercase. Normalize either.
    if s.eq_ignore_ascii_case("closed") {
        IssueState::Closed
    } else {
        IssueState::Open
    }
}

fn issue_ref_from_json(j: &GhIssueJson) -> IssueRef {
    IssueRef {
        id: IssueId::new(j.number.to_string()),
        title: j.title.clone(),
        state: parse_state(&j.state),
        labels: j.labels.iter().map(|l| l.name.clone()).collect(),
    }
}

fn parse_issue_number_from_url(url: &str) -> Option<String> {
    // gh issue create prints `https://github.com/<owner>/<repo>/issues/<number>`
    url.trim()
        .rsplit('/')
        .next()
        .filter(|s| s.chars().all(|c| c.is_ascii_digit()))
        .map(|s| s.to_string())
}

async fn run_gh(binary: &str, args: &[&str]) -> Result<String, BackendError> {
    let output = Command::new(binary)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| BackendError::Network(format!("spawn {binary}: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        // Heuristic: gh returns non-zero on auth failure with
        // "authentication" in stderr.
        if stderr.to_lowercase().contains("authent") {
            return Err(BackendError::Auth(stderr));
        }
        return Err(BackendError::Network(format!(
            "{binary} exited {:?}: {stderr}",
            output.status.code()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/github_backend.md#source
impl IssueBackend for GitHubIssueBackend {
    fn backend_kind(&self) -> BackendKind {
        BackendKind::GitHub
    }

    async fn create(&self, title: &str) -> Result<IssueId, BackendError> {
        let mut args: Vec<&str> = vec!["issue", "create", "--title", title, "--body", ""];
        if let Some(repo) = &self.repo {
            args.push("--repo");
            args.push(repo);
        }
        let stdout = run_gh(&self.binary, &args).await?;
        // gh issue create prints the issue URL on success; sometimes
        // also a banner line. Look for the first URL-shaped line and
        // pull the trailing number.
        for line in stdout.lines() {
            if let Some(num) = parse_issue_number_from_url(line) {
                return Ok(IssueId::new(num));
            }
        }
        Err(BackendError::Internal(format!(
            "gh issue create did not return a parseable URL; stdout was: {stdout:?}"
        )))
    }

    async fn list(&self, filter: &ListFilter) -> Result<Vec<IssueRef>, BackendError> {
        let state = match filter.state {
            IssueState::Open => "open",
            IssueState::Closed => "closed",
        };
        let mut args: Vec<&str> = vec![
            "issue",
            "list",
            "--json",
            "number,title,state,labels",
            "--state",
            state,
        ];
        // Each label is a separate --label flag.
        for label in &filter.labels {
            args.push("--label");
            args.push(label);
        }
        if let Some(repo) = &self.repo {
            args.push("--repo");
            args.push(repo);
        }
        let stdout = run_gh(&self.binary, &args).await?;
        let issues: Vec<GhIssueJson> = serde_json::from_str(&stdout)
            .map_err(|e| BackendError::Internal(format!("gh list JSON: {e}")))?;
        Ok(issues.iter().map(issue_ref_from_json).collect())
    }

    async fn read(&self, id: &IssueId) -> Result<IssueBody, BackendError> {
        let mut args: Vec<&str> = vec![
            "issue",
            "view",
            id.as_str(),
            "--json",
            "number,title,state,labels,body",
        ];
        if let Some(repo) = &self.repo {
            args.push("--repo");
            args.push(repo);
        }
        let stdout = run_gh(&self.binary, &args).await?;
        let j: GhIssueJson = serde_json::from_str(&stdout)
            .map_err(|e| BackendError::Internal(format!("gh view JSON: {e}")))?;
        // Surface a 404-shaped error specifically when gh returns
        // a not-found error; for slice 1 we just rely on run_gh's
        // network/auth split — closer-grained error mapping is a
        // follow-up.
        Ok(IssueBody {
            id: IssueId::new(j.number.to_string()),
            title: j.title.clone(),
            body_md: j.body.clone(),
            frontmatter: BTreeMap::new(),
        })
    }

    async fn update(&self, _id: &IssueId, _section: &str, _body: &str) -> Result<(), BackendError> {
        // R8: SDD CRRR fill semantics scoped to local in slice 1.
        Err(BackendError::Unsupported)
    }

    async fn close(&self, _id: &IssueId, _message: Option<&str>) -> Result<(), BackendError> {
        Err(BackendError::Unsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_state_lowercase_open() {
        assert_eq!(parse_state("open"), IssueState::Open);
    }

    #[test]
    fn parse_state_uppercase_closed() {
        assert_eq!(parse_state("CLOSED"), IssueState::Closed);
    }

    #[test]
    fn parse_state_unknown_defaults_open() {
        assert_eq!(parse_state("draft"), IssueState::Open);
    }

    #[test]
    fn issue_ref_from_json_extracts_label_names() {
        let j = GhIssueJson {
            number: 42,
            title: "fix it".into(),
            state: "OPEN".into(),
            labels: vec![
                GhLabel { name: "bug".into() },
                GhLabel { name: "p0".into() },
            ],
            body: String::new(),
        };
        let r = issue_ref_from_json(&j);
        assert_eq!(r.id.as_str(), "42");
        assert_eq!(r.state, IssueState::Open);
        assert_eq!(r.labels, vec!["bug", "p0"]);
    }

    #[test]
    fn parse_issue_number_from_url_happy() {
        assert_eq!(
            parse_issue_number_from_url("https://github.com/foo/bar/issues/42"),
            Some("42".into())
        );
    }

    #[test]
    fn parse_issue_number_from_url_trailing_newline() {
        assert_eq!(
            parse_issue_number_from_url("https://github.com/foo/bar/issues/42\n"),
            Some("42".into())
        );
    }

    #[test]
    fn parse_issue_number_from_url_non_numeric_suffix() {
        // Trailing path segment isn't a number — return None.
        assert_eq!(
            parse_issue_number_from_url("https://github.com/foo/bar/issues/abc"),
            None
        );
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
# GitHubIssueBackend is fully regenerable through the source template. The
# subprocess integration test file remains hand-written.
changes:
  - path: projects/agentic-workflow/src/runtime/github_backend.rs
    action: modify
    section: source
    impl_mode: codegen
    description: >
      Source template for GitHubIssueBackend, gh CLI subprocess invocation,
      issue JSON normalization, auth/network error mapping, and IssueBackend
      implementation. update() and close() remain Unsupported in slice 1.

  - path: projects/agentic-workflow/tests/github_backend_tests.rs
    action: create
    section: source
    impl_mode: hand-written
    description: >
      Integration tests for GitHubIssueBackend. Uses a mock gh executable
      (shell script written to a temp dir) to exercise the subprocess layer
      without network access. Covers: create_happy_path, auth_missing_returns_error,
      list_open_issues_maps_state, read_by_id_returns_body.
  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```
