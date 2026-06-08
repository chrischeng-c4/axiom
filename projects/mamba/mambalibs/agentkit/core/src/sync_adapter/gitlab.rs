//! GitLab sync adapter — syncs IssueProtocol, ChangeProtocol, and CodeIndexProtocol
//! to and from GitLab via its REST API v4.
//!
//! Supported: push_issue, pull_issue, push_change (MR), pull_code_index.
//! Not supported: push_spec — use [`ConfluenceSyncAdapter`] instead.

use super::{SyncAdapter, SyncResult};
use crate::error::{NovaError, NovaResult};
use crate::protocols::{
    ChangeProtocol, CodeIndexProtocol, IssuePriority, IssueProtocol, IssueStatus, SpecProtocol,
};
use async_trait::async_trait;
use mambalibs_http::client::{HttpClient, HttpMethod, RequestBuilder};
use serde::Deserialize;

/// GitLab sync adapter.
///
/// Supports: Issue (push + pull), Change via MR (push), Code index (pull).
/// `push_spec` returns `NotSupported` — use `ConfluenceSyncAdapter` instead.
///
/// Auth credentials are injected at construction time via a GitLab personal
/// access token.
pub struct GitLabSyncAdapter {
    client: HttpClient,
    token: String,
    base_url: String,
    project_id: String,
}

impl GitLabSyncAdapter {
    /// Create a new GitLab sync adapter.
    ///
    /// # Arguments
    /// * `token` — GitLab personal access token
    /// * `base_url` — GitLab instance base URL (e.g. `"https://gitlab.com"`)
    /// * `project_id` — Project ID or URL-encoded path (e.g. `"group/project"`)
    pub fn new(
        token: impl Into<String>,
        base_url: impl Into<String>,
        project_id: impl Into<String>,
    ) -> NovaResult<Self> {
        let client = HttpClient::default_client()
            .map_err(|e| NovaError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        let base_url = base_url.into().trim_end_matches('/').to_string();

        Ok(Self {
            client,
            token: token.into(),
            base_url,
            project_id: project_id.into(),
        })
    }

    fn api_url(&self, path: &str) -> String {
        let encoded = urlencoding::encode(&self.project_id);
        format!("{}/api/v4/projects/{}/{}", self.base_url, encoded, path)
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, url: &str) -> NovaResult<T> {
        let req = RequestBuilder::new(HttpMethod::Get, url)
            .header("PRIVATE-TOKEN", &self.token)
            .header("Accept", "application/json");

        let resp = self
            .client
            .execute_builder(req)
            .await
            .map_err(|e| NovaError::HttpError(e.to_string()))?;

        if !resp.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "GitLab {} {}: {}",
                resp.status_code, url, body
            )));
        }

        resp.json_as()
            .map_err(|e| NovaError::ApiError(format!("GitLab parse error: {}", e)))
    }

    async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> NovaResult<T> {
        let req = RequestBuilder::new(HttpMethod::Post, url)
            .header("PRIVATE-TOKEN", &self.token)
            .json_value(body);

        let resp = self
            .client
            .execute_builder(req)
            .await
            .map_err(|e| NovaError::HttpError(e.to_string()))?;

        if !resp.is_success() {
            let body_text = resp.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "GitLab {} {}: {}",
                resp.status_code, url, body_text
            )));
        }

        resp.json_as()
            .map_err(|e| NovaError::ApiError(format!("GitLab parse error: {}", e)))
    }
}

#[async_trait]
impl SyncAdapter for GitLabSyncAdapter {
    async fn push_issue(&self, issue: &IssueProtocol) -> NovaResult<SyncResult> {
        let url = self.api_url("issues");
        let body = serde_json::json!({
            "title": issue.title,
            "description": issue.description,
            "labels": issue.labels.join(","),
        });

        let created: GitLabIssueResponse = self.post(&url, body).await?;
        let issue_url = format!(
            "{}/{}/-/issues/{}",
            self.base_url, self.project_id, created.iid
        );

        Ok(SyncResult::created(created.iid.to_string(), issue_url))
    }

    async fn pull_issue(&self, external_id: &str) -> NovaResult<IssueProtocol> {
        let url = self.api_url(&format!("issues/{}", external_id));
        let gl: GitLabIssueResponse = self.get(&url).await?;

        Ok(IssueProtocol {
            id: gl.iid.to_string(),
            title: gl.title,
            description: gl.description.unwrap_or_default(),
            status: match gl.state.as_str() {
                "closed" => IssueStatus::Closed,
                _ => IssueStatus::Open,
            },
            priority: IssuePriority::default(),
            labels: gl.labels,
            acceptance_criteria: Vec::new(),
        })
    }

    async fn push_spec(&self, _spec: &SpecProtocol) -> NovaResult<SyncResult> {
        Err(NovaError::NotSupported(
            "GitLabSyncAdapter does not support spec sync; use ConfluenceSyncAdapter".into(),
        ))
    }

    async fn push_change(&self, change: &ChangeProtocol) -> NovaResult<SyncResult> {
        let branch = change.branch.as_deref().ok_or_else(|| {
            NovaError::InvalidArguments("ChangeProtocol.branch is required for push_change".into())
        })?;

        let url = self.api_url("merge_requests");
        let body = serde_json::json!({
            "title": format!("feat: {}", change.id),
            "source_branch": branch,
            "target_branch": "main",
            "description": format!(
                "Change: {}\nIssues: {}",
                change.id,
                change.issue_ids.join(", ")
            ),
        });

        let mr: GitLabMRResponse = self.post(&url, body).await?;
        Ok(SyncResult::created(mr.iid.to_string(), mr.web_url))
    }

    async fn pull_code_index(&self, path: &str) -> NovaResult<CodeIndexProtocol> {
        let encoded_path = urlencoding::encode(path);
        let url = self.api_url(&format!("repository/files/{}/raw?ref=main", encoded_path));

        let req = RequestBuilder::new(HttpMethod::Get, &url).header("PRIVATE-TOKEN", &self.token);

        let resp = self
            .client
            .execute_builder(req)
            .await
            .map_err(|e| NovaError::HttpError(e.to_string()))?;

        if !resp.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "GitLab file fetch {} {}: {}",
                resp.status_code, path, body
            )));
        }

        let content = resp.text().unwrap_or_default();

        // Try to deserialise as CodeIndexProtocol JSON; fall back to a minimal entry.
        if let Ok(index) = serde_json::from_str::<CodeIndexProtocol>(&content) {
            return Ok(index);
        }

        Ok(CodeIndexProtocol::new(path))
    }
}

// --- GitLab API response types ---

#[derive(Debug, Deserialize)]
struct GitLabIssueResponse {
    iid: u64,
    title: String,
    description: Option<String>,
    state: String,
    labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GitLabMRResponse {
    iid: u64,
    web_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_sync_adapter_new() {
        let adapter = GitLabSyncAdapter::new("token", "https://gitlab.com", "group/project");
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_gitlab_api_url() {
        let adapter =
            GitLabSyncAdapter::new("token", "https://gitlab.com", "group/project").unwrap();
        let url = adapter.api_url("issues");
        assert_eq!(
            url,
            "https://gitlab.com/api/v4/projects/group%2Fproject/issues"
        );
    }

    #[test]
    fn test_gitlab_api_url_trailing_slash() {
        let adapter =
            GitLabSyncAdapter::new("token", "https://gitlab.com/", "group/project").unwrap();
        let url = adapter.api_url("issues");
        assert!(url.starts_with("https://gitlab.com/api/v4/projects/"));
    }

    #[tokio::test]
    async fn test_push_spec_returns_not_supported() {
        let adapter =
            GitLabSyncAdapter::new("token", "https://gitlab.com", "group/project").unwrap();
        let spec = SpecProtocol::new("spec-1", ".aw/tech-design/foo.md", "# Spec");
        let result = adapter.push_spec(&spec).await;
        assert!(matches!(result, Err(NovaError::NotSupported(_))));
    }

    #[tokio::test]
    async fn test_push_change_without_branch_returns_error() {
        let adapter =
            GitLabSyncAdapter::new("token", "https://gitlab.com", "group/project").unwrap();
        let change = ChangeProtocol::new("my-change", "proj", vec!["1".into()]);
        // branch is None → should return InvalidArguments
        let result = adapter.push_change(&change).await;
        assert!(matches!(result, Err(NovaError::InvalidArguments(_))));
    }
}
