---
id: sdd-runtime-gitlab-backend
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

Public API manifest for `projects/agentic-workflow/src/runtime/gitlab_backend.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GitLabIssueBackend` | projects/agentic-workflow/src/runtime/gitlab_backend.rs | struct | pub | 29 |  |
| `from_env` | projects/agentic-workflow/src/runtime/gitlab_backend.rs | function | pub | 36 | from_env(repo: Option<String>) -> Result<Self, BackendError> |
| `with_binary` | projects/agentic-workflow/src/runtime/gitlab_backend.rs | function | pub | 45 | with_binary(mut self, binary: impl Into<String>) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: sdd-runtime-gitlab-backend-schema

definitions:
  GitLabIssueIid:
    description: >
      GitLab internal ID (iid) for an issue within a project — a positive
      integer serialised as a string to match the IssueId contract.
    type: string
    pattern: "^[1-9][0-9]*$"

  GlabIssueCreateArgs:
    description: >
      Arguments passed to `glab issue create` subprocess. `repo` is optional;
      when absent glab infers the project from the current git remote.
    type: object
    required: [title]
    properties:
      repo:
        type: string
        description: >
          `--repo NAMESPACE/PROJECT` passed to glab. Sourced from
          `[issue].gitlab_repo` in `.cue/config.toml` when present.
      title:
        type: string
        minLength: 1
      description:
        type: string
        description: Issue body markdown passed via `--description`.
        default: ""
    additionalProperties: false

  GlabIssueJsonFields:
    description: >
      Subset of fields in the JSON object produced by `glab issue list -F json`
      and `glab issue view <iid> -F json`. Only fields required for
      IssueRef and IssueBody are listed.
    type: object
    required: [iid, title, state]
    properties:
      iid:
        type: integer
        description: GitLab issue internal ID within the project (used as IssueId, stringified).
      title:
        type: string
      state:
        type: string
        enum: [opened, closed]
        description: >
          GitLab API state values. GitLabIssueBackend maps "opened" -> "open"
          and "closed" -> "closed" when constructing IssueRef / IssueBody.
      labels:
        type: array
        items: { type: string }
        description: GitLab labels are returned as plain strings (unlike GitHub objects).
        default: []
      description:
        type: string
        description: Full issue description markdown. Present in `view` output only.
    additionalProperties: true

  GlabListArgs:
    description: >
      Arguments for `glab issue list -F json`. State filter maps from
      ListFilter.state: open -> opened, closed -> closed, all -> all (glab values).
    type: object
    properties:
      state:
        type: string
        enum: [opened, closed, all]
        default: opened
      labels:
        type: array
        items: { type: string }
        description: >
          Each label passed as a separate `--label` flag. Empty list omits
          the flag (no label filtering).
        default: []
    additionalProperties: false

  GlabAuthConfig:
    description: >
      Authentication configuration for GitLabIssueBackend. Credentials are
      read-only at construction; no credential storage is performed (R10).
      The glab CLI reads GITLAB_TOKEN from the environment natively.
    type: object
    properties:
      token_env_var:
        type: string
        const: "GITLAB_TOKEN"
        description: >
          Name of the environment variable holding the GitLab personal access
          token. GitLabIssueBackend checks for its presence at construction
          and returns BackendError::Auth if absent.
      repo:
        type: string
        description: >
          Optional `NAMESPACE/PROJECT` override from `[issue].gitlab_repo` in
          `.cue/config.toml`. When absent glab infers from the git remote.
    additionalProperties: false
```
## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: gitlab_create_happy_path
    title: GitLabIssueBackend creates an issue via glab CLI subprocess and returns IssueId
    description: >
      When create(title) is called and a mock glab subprocess returns a valid
      JSON object with `iid`, the backend extracts the iid, stringifies it,
      and returns it as IssueId. The test replaces the glab binary path with
      a mock executable to avoid network access.
    given:
      - GITLAB_TOKEN env var is set to "test-token"
      - The mock glab binary is configured to print
        '{"iid": 17, "title": "new widget", "state": "opened", "labels": []}'
        on stdout with exit code 0 when given `issue create` args
    when:
      - GitLabIssueBackend::create("new widget") is called
    then:
      - The returned IssueId equals "17"
      - The subprocess was called with args including `issue create --title "new widget"`
    acceptance:
      - test: projects/agentic-workflow/tests/gitlab_backend_tests.rs::create_happy_path
        assertion: result == Ok(IssueId::from("17"))

  - id: gitlab_auth_missing
    title: GitLabIssueBackend returns BackendError::Auth when GITLAB_TOKEN is absent
    description: >
      GitLabIssueBackend checks for the GITLAB_TOKEN env var at construction
      time. When the var is absent, all five IssueBackend methods return
      Err(BackendError::Auth) without invoking the glab subprocess.
    given:
      - GITLAB_TOKEN env var is NOT set in the test environment
    when:
      - GitLabIssueBackend is constructed and create("title") is called
    then:
      - create returns Err(BackendError::Auth { message: contains "GITLAB_TOKEN" })
      - No subprocess was spawned
    acceptance:
      - test: projects/agentic-workflow/tests/gitlab_backend_tests.rs::auth_missing_returns_error
        assertion: matches!(err, BackendError::Auth { .. })
```
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/runtime/gitlab_backend.rs -->
```rust
//! GitLab Issues backend — shells out to the `glab` CLI.
//!
//! @spec projects/agentic-workflow/tech-design/core/logic/runtime/gitlab_backend.md
//!
//! Differences from `github_backend.rs`:
//! - Binary: `glab`, not `gh`
//! - Body flag: `--description`, not `--body`
//! - JSON flag: `-F json`, not `--json <fields>`
//! - Issue id field: `iid` (internal id within project), not `number`
//! - State values: `opened` / `closed` (lowercase)
//! - Labels: plain strings, not `{name: ...}` objects
//! - Auth env: `GITLAB_TOKEN`

use crate::runtime::issue_backend::{
    BackendError, BackendKind, IssueBackend, IssueBody, IssueId, IssueRef, IssueState, ListFilter,
};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::process::Stdio;
use tokio::process::Command;

const TOKEN_ENV_VAR: &str = "GITLAB_TOKEN";

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/gitlab_backend.md#changes
#[derive(Debug)]
pub struct GitLabIssueBackend {
    binary: String,
    repo: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/gitlab_backend.md#source
impl GitLabIssueBackend {
    pub fn from_env(repo: Option<String>) -> Result<Self, BackendError> {
        let _ = std::env::var(TOKEN_ENV_VAR)
            .map_err(|_| BackendError::Auth(format!("{TOKEN_ENV_VAR} env var is not set")))?;
        Ok(Self {
            binary: "glab".into(),
            repo,
        })
    }

    pub fn with_binary(mut self, binary: impl Into<String>) -> Self {
        self.binary = binary.into();
        self
    }
}

#[derive(Debug, Deserialize)]
struct GlabIssueJson {
    iid: u64,
    title: String,
    state: String,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    description: String,
}

fn parse_state(s: &str) -> IssueState {
    if s.eq_ignore_ascii_case("closed") {
        IssueState::Closed
    } else {
        IssueState::Open
    }
}

fn issue_ref_from_json(j: &GlabIssueJson) -> IssueRef {
    IssueRef {
        id: IssueId::new(j.iid.to_string()),
        title: j.title.clone(),
        state: parse_state(&j.state),
        labels: j.labels.clone(),
    }
}

fn parse_iid_from_url(url: &str) -> Option<String> {
    // glab issue create prints `https://gitlab.com/<group>/<proj>/-/issues/<iid>`
    url.trim()
        .rsplit('/')
        .next()
        .filter(|s| s.chars().all(|c| c.is_ascii_digit()))
        .map(|s| s.to_string())
}

async fn run_glab(binary: &str, args: &[&str]) -> Result<String, BackendError> {
    let output = Command::new(binary)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| BackendError::Network(format!("spawn {binary}: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        if stderr.to_lowercase().contains("authent") || stderr.to_lowercase().contains("token") {
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
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/gitlab_backend.md#source
impl IssueBackend for GitLabIssueBackend {
    fn backend_kind(&self) -> BackendKind {
        BackendKind::GitLab
    }

    async fn create(&self, title: &str) -> Result<IssueId, BackendError> {
        let mut args: Vec<&str> = vec!["issue", "create", "--title", title, "--description", ""];
        if let Some(repo) = &self.repo {
            args.push("--repo");
            args.push(repo);
        }
        let stdout = run_glab(&self.binary, &args).await?;
        for line in stdout.lines() {
            if let Some(iid) = parse_iid_from_url(line) {
                return Ok(IssueId::new(iid));
            }
        }
        Err(BackendError::Internal(format!(
            "glab issue create did not return a parseable URL; stdout was: {stdout:?}"
        )))
    }

    async fn list(&self, filter: &ListFilter) -> Result<Vec<IssueRef>, BackendError> {
        let state = match filter.state {
            IssueState::Open => "opened",
            IssueState::Closed => "closed",
        };
        let mut args: Vec<&str> = vec!["issue", "list", "-F", "json", "--state", state];
        for label in &filter.labels {
            args.push("--label");
            args.push(label);
        }
        if let Some(repo) = &self.repo {
            args.push("--repo");
            args.push(repo);
        }
        let stdout = run_glab(&self.binary, &args).await?;
        let issues: Vec<GlabIssueJson> = serde_json::from_str(&stdout)
            .map_err(|e| BackendError::Internal(format!("glab list JSON: {e}")))?;
        Ok(issues.iter().map(issue_ref_from_json).collect())
    }

    async fn read(&self, id: &IssueId) -> Result<IssueBody, BackendError> {
        let mut args: Vec<&str> = vec!["issue", "view", id.as_str(), "-F", "json"];
        if let Some(repo) = &self.repo {
            args.push("--repo");
            args.push(repo);
        }
        let stdout = run_glab(&self.binary, &args).await?;
        let j: GlabIssueJson = serde_json::from_str(&stdout)
            .map_err(|e| BackendError::Internal(format!("glab view JSON: {e}")))?;
        Ok(IssueBody {
            id: IssueId::new(j.iid.to_string()),
            title: j.title.clone(),
            body_md: j.description.clone(),
            frontmatter: BTreeMap::new(),
        })
    }

    async fn update(&self, _id: &IssueId, _section: &str, _body: &str) -> Result<(), BackendError> {
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
    fn parse_state_lowercase_opened() {
        assert_eq!(parse_state("opened"), IssueState::Open);
    }

    #[test]
    fn parse_state_lowercase_closed() {
        assert_eq!(parse_state("closed"), IssueState::Closed);
    }

    #[test]
    fn issue_ref_uses_iid_field() {
        let j = GlabIssueJson {
            iid: 42,
            title: "fix it".into(),
            state: "opened".into(),
            labels: vec!["bug".into(), "p0".into()],
            description: String::new(),
        };
        let r = issue_ref_from_json(&j);
        assert_eq!(r.id.as_str(), "42");
        assert_eq!(r.labels, vec!["bug", "p0"]);
    }

    #[test]
    fn parse_iid_gitlab_url() {
        assert_eq!(
            parse_iid_from_url("https://gitlab.com/group/proj/-/issues/13"),
            Some("13".into())
        );
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
# GitLabIssueBackend is fully regenerable through the source template. The
# subprocess integration test file remains hand-written.
changes:
  - path: projects/agentic-workflow/src/runtime/gitlab_backend.rs
    action: modify
    section: source
    impl_mode: codegen
    description: >
      Source template for GitLabIssueBackend, glab CLI subprocess invocation,
      issue JSON normalization, auth/network error mapping, and IssueBackend
      implementation. update() and close() remain Unsupported in slice 1.

  - path: projects/agentic-workflow/tests/gitlab_backend_tests.rs
    action: create
    section: source
    impl_mode: hand-written
    description: >
      Integration tests for GitLabIssueBackend. Uses a mock glab executable
      (shell script written to a temp dir) to exercise the subprocess layer
      without network access. Covers: create_happy_path, auth_missing_returns_error,
      list_open_issues_maps_state, read_by_iid_returns_body.
  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```
