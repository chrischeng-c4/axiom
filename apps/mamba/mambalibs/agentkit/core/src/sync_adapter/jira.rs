//! Jira sync adapter — syncs IssueProtocol to and from Jira Cloud (REST API v3).
//!
//! Supported: push_issue, pull_issue.
//! Not supported: push_spec, push_change, pull_code_index.

use super::{SyncAdapter, SyncResult};
use crate::error::{NovaError, NovaResult};
use crate::protocols::{
    ChangeProtocol, CodeIndexProtocol, IssuePriority, IssueProtocol, IssueStatus, SpecProtocol,
};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use mambalibs_http::client::{HttpClient, HttpMethod, RequestBuilder};
use serde::Deserialize;

/// Jira sync adapter.
///
/// Supports: Issue (push + pull) only.
/// Other operations return `NotSupported`.
///
/// Auth is HTTP Basic (`email:api_token`) injected at construction time.
pub struct JiraSyncAdapter {
    client: HttpClient,
    base_url: String,
    auth_header: String,
    project_key: String,
    issue_type: String,
}

impl JiraSyncAdapter {
    /// Create a new Jira sync adapter.
    ///
    /// # Arguments
    /// * `base_url` — Jira Cloud base URL (e.g. `"https://org.atlassian.net"`)
    /// * `email` — Jira account email
    /// * `api_token` — Jira API token
    /// * `project_key` — Target project key (e.g. `"PROJ"`)
    /// * `issue_type` — Default issue type name (e.g. `"Story"`)
    pub fn new(
        base_url: impl Into<String>,
        email: impl AsRef<str>,
        api_token: impl AsRef<str>,
        project_key: impl Into<String>,
        issue_type: impl Into<String>,
    ) -> NovaResult<Self> {
        let client = HttpClient::default_client()
            .map_err(|e| NovaError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        let credentials = format!("{}:{}", email.as_ref(), api_token.as_ref());
        let auth_header = format!("Basic {}", BASE64.encode(credentials));

        Ok(Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            auth_header,
            project_key: project_key.into(),
            issue_type: issue_type.into(),
        })
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}/rest/api/3/{}", self.base_url, path)
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, url: &str) -> NovaResult<T> {
        let req = RequestBuilder::new(HttpMethod::Get, url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json");

        let resp = self
            .client
            .execute_builder(req)
            .await
            .map_err(|e| NovaError::HttpError(e.to_string()))?;

        if !resp.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "Jira {} {}: {}",
                resp.status_code, url, body
            )));
        }

        resp.json_as()
            .map_err(|e| NovaError::ApiError(format!("Jira parse error: {}", e)))
    }

    async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> NovaResult<T> {
        let req = RequestBuilder::new(HttpMethod::Post, url)
            .header("Authorization", &self.auth_header)
            .json_value(body);

        let resp = self
            .client
            .execute_builder(req)
            .await
            .map_err(|e| NovaError::HttpError(e.to_string()))?;

        if !resp.is_success() {
            let body_text = resp.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "Jira {} {}: {}",
                resp.status_code, url, body_text
            )));
        }

        resp.json_as()
            .map_err(|e| NovaError::ApiError(format!("Jira parse error: {}", e)))
    }
}

#[async_trait]
impl SyncAdapter for JiraSyncAdapter {
    async fn push_issue(&self, issue: &IssueProtocol) -> NovaResult<SyncResult> {
        let url = self.api_url("issue");
        let body = serde_json::json!({
            "fields": {
                "project": { "key": self.project_key },
                "summary": issue.title,
                "description": {
                    "type": "doc",
                    "version": 1,
                    "content": [{
                        "type": "paragraph",
                        "content": [{ "type": "text", "text": issue.description }]
                    }]
                },
                "issuetype": { "name": self.issue_type }
            }
        });

        let created: JiraCreateResponse = self.post(&url, body).await?;
        let issue_url = format!("{}/browse/{}", self.base_url, created.key);

        Ok(SyncResult::created(created.key, issue_url))
    }

    async fn pull_issue(&self, external_id: &str) -> NovaResult<IssueProtocol> {
        let url = self.api_url(&format!("issue/{}", external_id));
        let jira: JiraIssue = self.get(&url).await?;

        let description = jira
            .fields
            .description
            .and_then(|d| {
                d.get("content")
                    .and_then(|c| c.get(0))
                    .and_then(|p| p.get("content"))
                    .and_then(|c| c.get(0))
                    .and_then(|t| t.get("text"))
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_default();

        let status = match jira.fields.status.name.as_str() {
            "Done" | "Closed" | "Resolved" => IssueStatus::Resolved,
            "In Progress" => IssueStatus::InProgress,
            _ => IssueStatus::Open,
        };

        Ok(IssueProtocol {
            id: jira.key,
            title: jira.fields.summary,
            description,
            status,
            priority: IssuePriority::default(),
            labels: jira.fields.labels.unwrap_or_default(),
            acceptance_criteria: Vec::new(),
        })
    }

    async fn push_spec(&self, _spec: &SpecProtocol) -> NovaResult<SyncResult> {
        Err(NovaError::NotSupported(
            "JiraSyncAdapter does not support spec sync; use ConfluenceSyncAdapter".into(),
        ))
    }

    async fn push_change(&self, _change: &ChangeProtocol) -> NovaResult<SyncResult> {
        Err(NovaError::NotSupported(
            "JiraSyncAdapter does not support change sync".into(),
        ))
    }

    async fn pull_code_index(&self, _path: &str) -> NovaResult<CodeIndexProtocol> {
        Err(NovaError::NotSupported(
            "JiraSyncAdapter does not support code index pull".into(),
        ))
    }
}

// --- Jira API response types ---

#[derive(Debug, Deserialize)]
struct JiraCreateResponse {
    key: String,
}

#[derive(Debug, Deserialize)]
struct JiraIssue {
    key: String,
    fields: JiraIssueFields,
}

#[derive(Debug, Deserialize)]
struct JiraIssueFields {
    summary: String,
    description: Option<serde_json::Value>,
    status: JiraStatus,
    labels: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct JiraStatus {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_adapter() -> JiraSyncAdapter {
        JiraSyncAdapter::new(
            "https://org.atlassian.net",
            "user@example.com",
            "api-token",
            "PROJ",
            "Story",
        )
        .unwrap()
    }

    #[test]
    fn test_jira_sync_adapter_new() {
        let adapter = JiraSyncAdapter::new(
            "https://org.atlassian.net",
            "user@example.com",
            "api-token",
            "PROJ",
            "Story",
        );
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_jira_api_url() {
        let adapter = make_adapter();
        let url = adapter.api_url("issue");
        assert_eq!(url, "https://org.atlassian.net/rest/api/3/issue");
    }

    #[tokio::test]
    async fn test_push_spec_returns_not_supported() {
        let spec = SpecProtocol::new("spec-1", ".aw/tech-design/foo.md", "# Spec");
        let result = make_adapter().push_spec(&spec).await;
        assert!(matches!(result, Err(NovaError::NotSupported(_))));
    }

    #[tokio::test]
    async fn test_push_change_returns_not_supported() {
        let change = ChangeProtocol::new("my-change", "proj", vec![]);
        let result = make_adapter().push_change(&change).await;
        assert!(matches!(result, Err(NovaError::NotSupported(_))));
    }

    #[tokio::test]
    async fn test_pull_code_index_returns_not_supported() {
        let result = make_adapter().pull_code_index("src/lib.rs").await;
        assert!(matches!(result, Err(NovaError::NotSupported(_))));
    }
}
