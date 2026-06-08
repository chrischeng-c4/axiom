// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/logic/runtime/jira_backend.md#source
// CODEGEN-BEGIN
//! Jira Issues backend — Atlassian Cloud REST API v3.
//!
//! @spec projects/agentic-workflow/tech-design/core/logic/runtime/jira_backend.md
//!
//! Auth: Basic `base64(email:token)`. Env vars:
//! - `JIRA_BASE_URL` — site URL (e.g. `https://acme.atlassian.net`)
//! - `JIRA_USER_EMAIL`
//! - `JIRA_API_TOKEN`
//! - `JIRA_PROJECT_KEY` — required for create() (REST endpoint needs it)
//!
//! Slice 1: create / list / read. update / close return
//! `BackendError::Unsupported` (R8). ADF wrapping for create's
//! description body is paragraph-only (R8 stretch goal: richer ADF).

use crate::runtime::issue_backend::{
    BackendError, BackendKind, IssueBackend, IssueBody, IssueId, IssueRef, IssueState, ListFilter,
};
use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;

const BASE_URL_ENV: &str = "JIRA_BASE_URL";
const EMAIL_ENV: &str = "JIRA_USER_EMAIL";
const TOKEN_ENV: &str = "JIRA_API_TOKEN";
const PROJECT_ENV: &str = "JIRA_PROJECT_KEY";

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/jira_backend.md#changes
#[derive(Debug)]
pub struct JiraIssueBackend {
    base_url: String,
    email: String,
    token: String,
    project_key: String,
    client: reqwest::Client,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/jira_backend.md#source
impl JiraIssueBackend {
    pub fn from_env() -> Result<Self, BackendError> {
        let base_url = std::env::var(BASE_URL_ENV)
            .map_err(|_| BackendError::Auth(format!("{BASE_URL_ENV} not set")))?;
        let email = std::env::var(EMAIL_ENV)
            .map_err(|_| BackendError::Auth(format!("{EMAIL_ENV} not set")))?;
        let token = std::env::var(TOKEN_ENV)
            .map_err(|_| BackendError::Auth(format!("{TOKEN_ENV} not set")))?;
        let project_key = std::env::var(PROJECT_ENV)
            .map_err(|_| BackendError::Auth(format!("{PROJECT_ENV} not set")))?;
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| BackendError::Internal(format!("reqwest client: {e}")))?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            email,
            token,
            project_key,
            client,
        })
    }

    /// Test override — point at a mock HTTP server (e.g. wiremock).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
        self
    }

    fn auth_header(&self) -> String {
        let raw = format!("{}:{}", self.email, self.token);
        format!("Basic {}", B64.encode(raw.as_bytes()))
    }
}

/// Wrap a single-paragraph plain-text body in minimal ADF. Slice 1
/// stretch: just enough to satisfy the API's description requirement.
fn adf_paragraph(text: &str) -> serde_json::Value {
    json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "paragraph",
                "content": [
                    { "type": "text", "text": text }
                ]
            }
        ]
    })
}

/// Best-effort flatten an ADF document back to plain text — slice 1
/// just walks the tree picking up `text` nodes. Richer reverse-conversion
/// (preserving headings / lists / code) is a follow-up.
fn flatten_adf_to_text(adf: &serde_json::Value) -> String {
    let mut out = String::new();
    flatten_adf_node(adf, &mut out);
    out
}

fn flatten_adf_node(node: &serde_json::Value, out: &mut String) {
    if let Some(text) = node.get("text").and_then(|v| v.as_str()) {
        out.push_str(text);
    }
    if let Some(content) = node.get("content").and_then(|v| v.as_array()) {
        for child in content {
            flatten_adf_node(child, out);
        }
        // After a paragraph, insert a newline for readability.
        if node.get("type").and_then(|v| v.as_str()) == Some("paragraph") {
            out.push('\n');
        }
    }
}

#[derive(Debug, Deserialize)]
struct JiraCreateResponse {
    key: String,
}

#[derive(Debug, Deserialize)]
struct JiraSearchResponse {
    issues: Vec<JiraIssueResponse>,
}

#[derive(Debug, Deserialize)]
struct JiraIssueResponse {
    key: String,
    fields: JiraFields,
}

#[derive(Debug, Deserialize)]
struct JiraFields {
    summary: String,
    status: JiraStatus,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    description: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct JiraStatus {
    #[serde(rename = "statusCategory")]
    status_category: JiraStatusCategory,
}

#[derive(Debug, Deserialize)]
struct JiraStatusCategory {
    key: String,
}

fn map_status(category: &str) -> IssueState {
    if category.eq_ignore_ascii_case("done") {
        IssueState::Closed
    } else {
        // "new" / "indeterminate" / unknown all map to Open.
        IssueState::Open
    }
}

fn issue_ref_from_response(r: &JiraIssueResponse) -> IssueRef {
    IssueRef {
        id: IssueId::new(r.key.clone()),
        title: r.fields.summary.clone(),
        state: map_status(&r.fields.status.status_category.key),
        labels: r.fields.labels.clone(),
    }
}

fn build_jql(filter: &ListFilter, project_key: &str) -> String {
    let mut parts: Vec<String> = vec![format!("project={project_key}")];
    match filter.state {
        IssueState::Open => parts.push("statusCategory!=Done".to_string()),
        IssueState::Closed => parts.push("statusCategory=Done".to_string()),
    }
    for label in &filter.labels {
        parts.push(format!("labels=\"{}\"", label.replace('"', "\\\"")));
    }
    parts.join(" AND ")
}

async fn map_response_status(resp: reqwest::Response) -> Result<reqwest::Response, BackendError> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let body = resp.text().await.unwrap_or_default();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err(BackendError::Auth(format!("{status}: {body}")));
    }
    if status == reqwest::StatusCode::NOT_FOUND {
        // Caller knows which IssueId triggered this — surface generic
        // for slice 1; richer mapping via per-method wrappers is a follow-up.
        return Err(BackendError::Network(format!("404: {body}")));
    }
    Err(BackendError::Network(format!("{status}: {body}")))
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/jira_backend.md#source
impl IssueBackend for JiraIssueBackend {
    fn backend_kind(&self) -> BackendKind {
        BackendKind::Jira
    }

    async fn create(&self, title: &str) -> Result<IssueId, BackendError> {
        let url = format!("{}/rest/api/3/issue", self.base_url);
        let body = json!({
            "fields": {
                "project": { "key": self.project_key },
                "summary": title,
                "issuetype": { "name": "Task" },
                "description": adf_paragraph(""),
            }
        });
        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| BackendError::Network(format!("POST {url}: {e}")))?;
        let resp = map_response_status(resp).await?;
        let parsed: JiraCreateResponse = resp
            .json()
            .await
            .map_err(|e| BackendError::Internal(format!("create JSON: {e}")))?;
        Ok(IssueId::new(parsed.key))
    }

    async fn list(&self, filter: &ListFilter) -> Result<Vec<IssueRef>, BackendError> {
        let url = format!("{}/rest/api/3/search/jql", self.base_url);
        let jql = build_jql(filter, &self.project_key);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .query(&[("jql", jql.as_str()), ("fields", "summary,status,labels")])
            .send()
            .await
            .map_err(|e| BackendError::Network(format!("GET {url}: {e}")))?;
        let resp = map_response_status(resp).await?;
        let parsed: JiraSearchResponse = resp
            .json()
            .await
            .map_err(|e| BackendError::Internal(format!("list JSON: {e}")))?;
        Ok(parsed.issues.iter().map(issue_ref_from_response).collect())
    }

    async fn read(&self, id: &IssueId) -> Result<IssueBody, BackendError> {
        let url = format!("{}/rest/api/3/issue/{}", self.base_url, id.as_str());
        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| BackendError::Network(format!("GET {url}: {e}")))?;
        let resp = map_response_status(resp).await?;
        let parsed: JiraIssueResponse = resp
            .json()
            .await
            .map_err(|e| BackendError::Internal(format!("read JSON: {e}")))?;
        let body_md = parsed
            .fields
            .description
            .as_ref()
            .map(flatten_adf_to_text)
            .unwrap_or_default();
        Ok(IssueBody {
            id: IssueId::new(parsed.key.clone()),
            title: parsed.fields.summary.clone(),
            body_md,
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
    fn adf_paragraph_shape() {
        let v = adf_paragraph("hello");
        assert_eq!(v["type"], "doc");
        assert_eq!(v["version"], 1);
        assert_eq!(v["content"][0]["type"], "paragraph");
        assert_eq!(v["content"][0]["content"][0]["text"], "hello");
    }

    #[test]
    fn flatten_adf_picks_text_nodes() {
        let v = json!({
            "type": "doc",
            "content": [
                { "type": "paragraph", "content": [
                    { "type": "text", "text": "Hello " },
                    { "type": "text", "text": "world." },
                ]},
                { "type": "paragraph", "content": [
                    { "type": "text", "text": "Second." },
                ]},
            ]
        });
        assert_eq!(flatten_adf_to_text(&v), "Hello world.\nSecond.\n");
    }

    #[test]
    fn build_jql_open_with_labels() {
        let f = ListFilter {
            state: IssueState::Open,
            labels: vec!["bug".into(), "p0".into()],
        };
        let jql = build_jql(&f, "PROJ");
        assert!(jql.contains("project=PROJ"));
        assert!(jql.contains("statusCategory!=Done"));
        assert!(jql.contains("labels=\"bug\""));
        assert!(jql.contains("labels=\"p0\""));
    }

    #[test]
    fn build_jql_closed() {
        let f = ListFilter {
            state: IssueState::Closed,
            labels: vec![],
        };
        let jql = build_jql(&f, "X");
        assert!(jql.contains("statusCategory=Done"));
    }

    #[test]
    fn map_status_done_is_closed() {
        assert_eq!(map_status("done"), IssueState::Closed);
        assert_eq!(map_status("Done"), IssueState::Closed);
    }

    #[test]
    fn map_status_other_is_open() {
        assert_eq!(map_status("new"), IssueState::Open);
        assert_eq!(map_status("indeterminate"), IssueState::Open);
        assert_eq!(map_status("anything"), IssueState::Open);
    }
}

// CODEGEN-END
