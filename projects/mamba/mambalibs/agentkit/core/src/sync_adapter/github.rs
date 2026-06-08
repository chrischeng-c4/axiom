//! GitHub sync adapter — syncs IssueProtocol, ChangeProtocol, and CodeIndexProtocol
//! to and from GitHub via its REST API v3.
//!
//! Supported: push_issue, pull_issue, push_change (PR), pull_code_index.
//! Not supported: push_spec — use [`GDocsSyncAdapter`] instead.

use super::{SyncAdapter, SyncResult};
use crate::error::{NovaError, NovaResult};
use crate::protocols::{
    ChangeProtocol, CodeIndexProtocol, IssuePriority, IssueProtocol, IssueStatus, SpecProtocol,
};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use mambalibs_http::client::{HttpClient, HttpMethod, RequestBuilder};
use serde::Deserialize;

/// GitHub sync adapter.
///
/// Supports: Issue (push + pull), Change via PR (push), Code index (pull).
/// `push_spec` returns `NotSupported` — use `GDocsSyncAdapter` instead.
///
/// Auth credentials are injected at construction time via a GitHub personal
/// access token.
pub struct GitHubSyncAdapter {
    client: HttpClient,
    token: String,
    owner: String,
    repo: String,
}

impl GitHubSyncAdapter {
    /// Create a new GitHub sync adapter.
    ///
    /// # Arguments
    /// * `token` — GitHub personal access token
    /// * `owner` — Repository owner (user or organisation)
    /// * `repo` — Repository name
    pub fn new(
        token: impl Into<String>,
        owner: impl Into<String>,
        repo: impl Into<String>,
    ) -> NovaResult<Self> {
        let client = HttpClient::default_client()
            .map_err(|e| NovaError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            token: token.into(),
            owner: owner.into(),
            repo: repo.into(),
        })
    }

    fn api_url(&self, path: &str) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/{}",
            self.owner, self.repo, path
        )
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, url: &str) -> NovaResult<T> {
        let req = RequestBuilder::new(HttpMethod::Get, url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "cclab-nova");

        let resp = self
            .client
            .execute_builder(req)
            .await
            .map_err(|e| NovaError::HttpError(e.to_string()))?;

        if !resp.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "GitHub {} {}: {}",
                resp.status_code, url, body
            )));
        }

        resp.json_as()
            .map_err(|e| NovaError::ApiError(format!("GitHub parse error: {}", e)))
    }

    async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> NovaResult<T> {
        let req = RequestBuilder::new(HttpMethod::Post, url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "cclab-nova")
            .json_value(body);

        let resp = self
            .client
            .execute_builder(req)
            .await
            .map_err(|e| NovaError::HttpError(e.to_string()))?;

        if !resp.is_success() {
            let body_text = resp.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "GitHub {} {}: {}",
                resp.status_code, url, body_text
            )));
        }

        resp.json_as()
            .map_err(|e| NovaError::ApiError(format!("GitHub parse error: {}", e)))
    }
}

#[async_trait]
impl SyncAdapter for GitHubSyncAdapter {
    async fn push_issue(&self, issue: &IssueProtocol) -> NovaResult<SyncResult> {
        let url = self.api_url("issues");
        let body = serde_json::json!({
            "title": issue.title,
            "body": issue.description,
            "labels": issue.labels,
        });

        let created: GitHubIssueResponse = self.post(&url, body).await?;
        Ok(SyncResult::created(
            created.number.to_string(),
            created.html_url,
        ))
    }

    async fn pull_issue(&self, external_id: &str) -> NovaResult<IssueProtocol> {
        let url = self.api_url(&format!("issues/{}", external_id));
        let gh: GitHubIssueResponse = self.get(&url).await?;

        Ok(IssueProtocol {
            id: gh.number.to_string(),
            title: gh.title,
            description: gh.body.unwrap_or_default(),
            status: match gh.state.as_str() {
                "closed" => IssueStatus::Closed,
                _ => IssueStatus::Open,
            },
            priority: IssuePriority::default(),
            labels: gh.labels.into_iter().map(|l| l.name).collect(),
            acceptance_criteria: Vec::new(),
        })
    }

    async fn push_spec(&self, _spec: &SpecProtocol) -> NovaResult<SyncResult> {
        Err(NovaError::NotSupported(
            "GitHubSyncAdapter does not support spec sync; use GDocsSyncAdapter".into(),
        ))
    }

    async fn push_change(&self, change: &ChangeProtocol) -> NovaResult<SyncResult> {
        let branch = change.branch.as_deref().ok_or_else(|| {
            NovaError::InvalidArguments("ChangeProtocol.branch is required for push_change".into())
        })?;

        let url = self.api_url("pulls");
        let body = serde_json::json!({
            "title": format!("feat: {}", change.id),
            "head": branch,
            "base": "main",
            "body": format!(
                "Change: {}\nIssues: {}",
                change.id,
                change.issue_ids.join(", ")
            ),
        });

        let pr: GitHubPRResponse = self.post(&url, body).await?;
        Ok(SyncResult::created(pr.number.to_string(), pr.html_url))
    }

    async fn pull_code_index(&self, path: &str) -> NovaResult<CodeIndexProtocol> {
        let url = self.api_url(&format!("contents/{}", path));

        let req = RequestBuilder::new(HttpMethod::Get, &url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "cclab-nova");

        let resp = self
            .client
            .execute_builder(req)
            .await
            .map_err(|e| NovaError::HttpError(e.to_string()))?;

        if !resp.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "GitHub contents {} {}: {}",
                resp.status_code, path, body
            )));
        }

        let file: GitHubContentsResponse = resp
            .json_as()
            .map_err(|e| NovaError::ApiError(format!("GitHub parse error: {}", e)))?;

        let content_bytes = BASE64
            .decode(file.content.replace('\n', ""))
            .map_err(|e| NovaError::ApiError(format!("Base64 decode error: {}", e)))?;
        let content = String::from_utf8(content_bytes)
            .map_err(|e| NovaError::ApiError(format!("UTF-8 decode error: {}", e)))?;

        // Try to deserialise as CodeIndexProtocol JSON; fall back to a minimal entry.
        if let Ok(index) = serde_json::from_str::<CodeIndexProtocol>(&content) {
            return Ok(index);
        }

        Ok(CodeIndexProtocol::new(path))
    }
}

// --- GitHub API response types ---

#[derive(Debug, Deserialize)]
struct GitHubLabel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GitHubIssueResponse {
    number: u64,
    title: String,
    body: Option<String>,
    state: String,
    html_url: String,
    labels: Vec<GitHubLabel>,
}

#[derive(Debug, Deserialize)]
struct GitHubPRResponse {
    number: u64,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubContentsResponse {
    content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_sync_adapter_new() {
        let adapter = GitHubSyncAdapter::new("token", "owner", "repo");
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_github_api_url() {
        let adapter = GitHubSyncAdapter::new("token", "myorg", "myrepo").unwrap();
        let url = adapter.api_url("issues");
        assert_eq!(url, "https://api.github.com/repos/myorg/myrepo/issues");
    }

    #[tokio::test]
    async fn test_push_spec_returns_not_supported() {
        let adapter = GitHubSyncAdapter::new("token", "owner", "repo").unwrap();
        let spec = SpecProtocol::new("spec-1", ".aw/tech-design/foo.md", "# Spec");
        let result = adapter.push_spec(&spec).await;
        assert!(matches!(result, Err(NovaError::NotSupported(_))));
    }

    #[tokio::test]
    async fn test_push_change_without_branch_returns_error() {
        let adapter = GitHubSyncAdapter::new("token", "owner", "repo").unwrap();
        let change = ChangeProtocol::new("my-change", "proj", vec!["1".into()]);
        let result = adapter.push_change(&change).await;
        assert!(matches!(result, Err(NovaError::InvalidArguments(_))));
    }
}
