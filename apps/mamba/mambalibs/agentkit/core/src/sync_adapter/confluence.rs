//! Confluence sync adapter — syncs SpecProtocol to Confluence Cloud (REST API).
//!
//! Supported: push_spec.
//! Not supported: push_issue, pull_issue, push_change, pull_code_index.

use super::{SyncAdapter, SyncResult};
use crate::error::{NovaError, NovaResult};
use crate::protocols::{ChangeProtocol, CodeIndexProtocol, IssueProtocol, SpecProtocol};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use mambalibs_http::client::{HttpClient, HttpMethod, RequestBuilder};
use serde::Deserialize;

/// Confluence sync adapter.
///
/// Supports: Spec (push) only.
/// Other operations return `NotSupported`.
///
/// Auth is HTTP Basic (`email:api_token`) injected at construction time.
pub struct ConfluenceSyncAdapter {
    client: HttpClient,
    base_url: String,
    auth_header: String,
    space_key: String,
}

impl ConfluenceSyncAdapter {
    /// Create a new Confluence sync adapter.
    ///
    /// # Arguments
    /// * `base_url` — Confluence base URL (e.g. `"https://org.atlassian.net/wiki"`)
    /// * `email` — Atlassian account email
    /// * `api_token` — Atlassian API token
    /// * `space_key` — Target Confluence space key (e.g. `"ENG"`)
    pub fn new(
        base_url: impl Into<String>,
        email: impl AsRef<str>,
        api_token: impl AsRef<str>,
        space_key: impl Into<String>,
    ) -> NovaResult<Self> {
        let client = HttpClient::default_client()
            .map_err(|e| NovaError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        let credentials = format!("{}:{}", email.as_ref(), api_token.as_ref());
        let auth_header = format!("Basic {}", BASE64.encode(credentials));

        Ok(Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            auth_header,
            space_key: space_key.into(),
        })
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}/rest/api/{}", self.base_url, path)
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
                "Confluence {} {}: {}",
                resp.status_code, url, body_text
            )));
        }

        resp.json_as()
            .map_err(|e| NovaError::ApiError(format!("Confluence parse error: {}", e)))
    }
}

#[async_trait]
impl SyncAdapter for ConfluenceSyncAdapter {
    async fn push_issue(&self, _issue: &IssueProtocol) -> NovaResult<SyncResult> {
        Err(NovaError::NotSupported(
            "ConfluenceSyncAdapter does not support issue sync; use JiraSyncAdapter".into(),
        ))
    }

    async fn pull_issue(&self, _external_id: &str) -> NovaResult<IssueProtocol> {
        Err(NovaError::NotSupported(
            "ConfluenceSyncAdapter does not support issue pull".into(),
        ))
    }

    async fn push_spec(&self, spec: &SpecProtocol) -> NovaResult<SyncResult> {
        let url = self.api_url("content");

        let title = spec
            .id
            .split('/')
            .last()
            .unwrap_or(&spec.id)
            .replace('-', " ")
            .replace('_', " ");

        let body = serde_json::json!({
            "type": "page",
            "title": title,
            "space": { "key": self.space_key },
            "body": {
                "storage": {
                    "value": format!(
                        "<p>{}</p>",
                        html_escape::encode_text(&spec.content)
                    ),
                    "representation": "storage"
                }
            }
        });

        let created: ConfluencePageResponse = self.post(&url, body).await?;
        let page_url = format!("{}/pages/{}", self.base_url, created.id);

        Ok(SyncResult::created(created.id, page_url))
    }

    async fn push_change(&self, _change: &ChangeProtocol) -> NovaResult<SyncResult> {
        Err(NovaError::NotSupported(
            "ConfluenceSyncAdapter does not support change sync".into(),
        ))
    }

    async fn pull_code_index(&self, _path: &str) -> NovaResult<CodeIndexProtocol> {
        Err(NovaError::NotSupported(
            "ConfluenceSyncAdapter does not support code index pull".into(),
        ))
    }
}

// --- Confluence API response types ---

#[derive(Debug, Deserialize)]
struct ConfluencePageResponse {
    id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_adapter() -> ConfluenceSyncAdapter {
        ConfluenceSyncAdapter::new(
            "https://org.atlassian.net/wiki",
            "user@example.com",
            "api-token",
            "ENG",
        )
        .unwrap()
    }

    #[test]
    fn test_confluence_sync_adapter_new() {
        let adapter = ConfluenceSyncAdapter::new(
            "https://org.atlassian.net/wiki",
            "user@example.com",
            "api-token",
            "ENG",
        );
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_confluence_api_url() {
        let adapter = make_adapter();
        let url = adapter.api_url("content");
        assert_eq!(url, "https://org.atlassian.net/wiki/rest/api/content");
    }

    #[tokio::test]
    async fn test_push_issue_returns_not_supported() {
        let issue = IssueProtocol::new("1", "Test issue");
        let result = make_adapter().push_issue(&issue).await;
        assert!(matches!(result, Err(NovaError::NotSupported(_))));
    }

    #[tokio::test]
    async fn test_pull_issue_returns_not_supported() {
        let result = make_adapter().pull_issue("123").await;
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
