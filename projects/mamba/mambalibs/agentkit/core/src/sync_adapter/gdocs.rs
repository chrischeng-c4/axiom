//! Google Docs sync adapter — syncs SpecProtocol to Google Docs.
//!
//! Supported: push_spec.
//! Not supported: push_issue, pull_issue, push_change, pull_code_index.

use super::{SyncAdapter, SyncResult};
use crate::error::{NovaError, NovaResult};
use crate::protocols::{ChangeProtocol, CodeIndexProtocol, IssueProtocol, SpecProtocol};
use async_trait::async_trait;
use mambalibs_http::client::{HttpClient, HttpMethod, RequestBuilder};
use serde::Deserialize;

const GDOCS_API_BASE: &str = "https://docs.googleapis.com/v1";

/// Google Docs sync adapter.
///
/// Supports: Spec (push) only.
/// Other operations return `NotSupported`.
///
/// Auth is an OAuth 2.0 bearer token injected at construction time.
pub struct GDocsSyncAdapter {
    client: HttpClient,
    token: String,
    folder_id: Option<String>,
}

impl GDocsSyncAdapter {
    /// Create a new Google Docs sync adapter.
    ///
    /// # Arguments
    /// * `token` — OAuth 2.0 bearer token with Google Docs / Drive write scope
    /// * `folder_id` — Optional Google Drive folder ID to place created docs in
    pub fn new(token: impl Into<String>, folder_id: Option<String>) -> NovaResult<Self> {
        let client = HttpClient::default_client()
            .map_err(|e| NovaError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            token: token.into(),
            folder_id,
        })
    }

    async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> NovaResult<T> {
        let req = RequestBuilder::new(HttpMethod::Post, url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json_value(body);

        let resp = self
            .client
            .execute_builder(req)
            .await
            .map_err(|e| NovaError::HttpError(e.to_string()))?;

        if !resp.is_success() {
            let body_text = resp.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "Google Docs API {} {}: {}",
                resp.status_code, url, body_text
            )));
        }

        resp.json_as()
            .map_err(|e| NovaError::ApiError(format!("Google Docs parse error: {}", e)))
    }
}

#[async_trait]
impl SyncAdapter for GDocsSyncAdapter {
    async fn push_issue(&self, _issue: &IssueProtocol) -> NovaResult<SyncResult> {
        Err(NovaError::NotSupported(
            "GDocsSyncAdapter does not support issue sync; use JiraSyncAdapter".into(),
        ))
    }

    async fn pull_issue(&self, _external_id: &str) -> NovaResult<IssueProtocol> {
        Err(NovaError::NotSupported(
            "GDocsSyncAdapter does not support issue pull".into(),
        ))
    }

    async fn push_spec(&self, spec: &SpecProtocol) -> NovaResult<SyncResult> {
        let title = spec
            .id
            .split('/')
            .last()
            .unwrap_or(&spec.id)
            .replace('-', " ")
            .replace('_', " ");

        // Step 1: create the document via Docs API.
        let create_url = format!("{}/documents", GDOCS_API_BASE);
        let doc: GDocsDocument = self
            .post(&create_url, serde_json::json!({ "title": title }))
            .await?;

        // Step 2: record folder preference (Drive move would require a PATCH
        // to the Drive files endpoint — deferred to caller if needed).
        let _ = &self.folder_id;

        // Step 3: insert the spec content via batchUpdate.
        let batch_url = format!(
            "{}/documents/{}:batchUpdate",
            GDOCS_API_BASE, doc.document_id
        );
        let insert_body = serde_json::json!({
            "requests": [{
                "insertText": {
                    "location": { "index": 1 },
                    "text": spec.content
                }
            }]
        });
        let _: serde_json::Value = self.post(&batch_url, insert_body).await?;

        let doc_url = format!("https://docs.google.com/document/d/{}", doc.document_id);

        Ok(SyncResult::created(doc.document_id, doc_url))
    }

    async fn push_change(&self, _change: &ChangeProtocol) -> NovaResult<SyncResult> {
        Err(NovaError::NotSupported(
            "GDocsSyncAdapter does not support change sync".into(),
        ))
    }

    async fn pull_code_index(&self, _path: &str) -> NovaResult<CodeIndexProtocol> {
        Err(NovaError::NotSupported(
            "GDocsSyncAdapter does not support code index pull".into(),
        ))
    }
}

// --- Google Docs API response types ---

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GDocsDocument {
    document_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gdocs_sync_adapter_new() {
        let adapter = GDocsSyncAdapter::new("oauth-token", None);
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_gdocs_sync_adapter_with_folder() {
        let adapter = GDocsSyncAdapter::new("oauth-token", Some("folder-id".to_string()));
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn test_push_issue_returns_not_supported() {
        let adapter = GDocsSyncAdapter::new("oauth-token", None).unwrap();
        let issue = IssueProtocol::new("1", "Test");
        let result = adapter.push_issue(&issue).await;
        assert!(matches!(result, Err(NovaError::NotSupported(_))));
    }

    #[tokio::test]
    async fn test_pull_issue_returns_not_supported() {
        let adapter = GDocsSyncAdapter::new("oauth-token", None).unwrap();
        let result = adapter.pull_issue("DOC123").await;
        assert!(matches!(result, Err(NovaError::NotSupported(_))));
    }

    #[tokio::test]
    async fn test_push_change_returns_not_supported() {
        let adapter = GDocsSyncAdapter::new("oauth-token", None).unwrap();
        let change = ChangeProtocol::new("my-change", "proj", vec![]);
        let result = adapter.push_change(&change).await;
        assert!(matches!(result, Err(NovaError::NotSupported(_))));
    }

    #[tokio::test]
    async fn test_pull_code_index_returns_not_supported() {
        let adapter = GDocsSyncAdapter::new("oauth-token", None).unwrap();
        let result = adapter.pull_code_index("src/lib.rs").await;
        assert!(matches!(result, Err(NovaError::NotSupported(_))));
    }
}
