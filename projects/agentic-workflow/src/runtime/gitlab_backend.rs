// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/logic/runtime/gitlab_backend.md#source
// CODEGEN-BEGIN
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
        let mut args: Vec<&str> = vec!["issue", "list", "--output", "json"];
        if matches!(filter.state, IssueState::Closed) {
            args.push("--closed");
        }
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

// CODEGEN-END
