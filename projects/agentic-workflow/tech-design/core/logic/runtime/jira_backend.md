---
id: sdd-runtime-jira-backend
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

Public API manifest for `projects/agentic-workflow/src/runtime/jira_backend.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `JiraIssueBackend` | projects/agentic-workflow/src/runtime/jira_backend.rs | struct | pub | 34 |  |
| `from_env` | projects/agentic-workflow/src/runtime/jira_backend.rs | function | pub | 44 | from_env() -> Result<Self, BackendError> |
| `with_base_url` | projects/agentic-workflow/src/runtime/jira_backend.rs | function | pub | 66 | with_base_url(mut self, base_url: impl Into<String>) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: sdd-runtime-jira-backend-schema

definitions:
  JiraIssueKey:
    description: >
      Jira issue key — `<PROJECT>-<NUMBER>` (e.g. "PROJ-456"). Used as
      IssueId on this backend.
    type: string
    pattern: "^[A-Z][A-Z0-9_]+-[1-9][0-9]*$"

  JiraAuthConfig:
    description: >
      Authentication for Jira REST API v3. Basic auth with the user's
      email + an API token. Base URL identifies the Atlassian Cloud site
      (e.g. https://acme.atlassian.net). Project key namespaces issue
      operations.
    type: object
    required: [base_url_env, email_env, token_env]
    properties:
      base_url_env:
        type: string
        const: "JIRA_BASE_URL"
      email_env:
        type: string
        const: "JIRA_USER_EMAIL"
      token_env:
        type: string
        const: "JIRA_API_TOKEN"
      project_env:
        type: string
        const: "JIRA_PROJECT_KEY"
        description: >
          Default project key for create() — required since the REST
          create endpoint needs a project. Sourced from JIRA_PROJECT_KEY
          env var (or `[issue].jira_project` config in a future slice).
    additionalProperties: false

  JiraCreateRequest:
    description: >
      Body shape for `POST /rest/api/3/issue`. Description must be in
      ADF (Atlassian Document Format) — paragraph-only minimum is enough
      for slice 1.
    type: object
    required: [fields]
    properties:
      fields:
        type: object
        required: [project, summary, issuetype]
        properties:
          project:
            type: object
            required: [key]
            properties:
              key: { type: string }
          summary:
            type: string
            minLength: 1
          issuetype:
            type: object
            required: [name]
            properties:
              name:
                type: string
                default: "Task"
          description:
            $ref: "#/definitions/AdfDocument"

  AdfDocument:
    description: >
      Minimal Atlassian Document Format envelope — paragraphs only.
      Slice 1 wraps the issue body in a single paragraph with a single
      text node. Richer ADF (lists, code blocks, mentions) is a follow-up.
    type: object
    required: [type, version, content]
    properties:
      type: { type: string, const: "doc" }
      version: { type: integer, const: 1 }
      content:
        type: array
        items:
          type: object
          required: [type, content]
          properties:
            type: { type: string, const: "paragraph" }
            content:
              type: array
              items:
                type: object
                required: [type, text]
                properties:
                  type: { type: string, const: "text" }
                  text: { type: string }

  JiraCreateResponse:
    description: >
      Subset of fields from `POST /rest/api/3/issue` response. Only
      `key` is required by the backend (becomes the IssueId).
    type: object
    required: [key]
    properties:
      key: { type: string }
      id: { type: string }

  JiraSearchRequest:
    description: >
      Body shape for `GET /rest/api/3/search/jql`. JQL composed from
      ListFilter — project=<key> AND status[Category]=<state> [AND
      labels=<l> [AND labels=<l>...]].
    type: object
    required: [jql]
    properties:
      jql: { type: string }
      maxResults: { type: integer, default: 50 }
      fields:
        type: array
        items: { type: string }
        default: [summary, status, labels]

  JiraIssueResponse:
    description: >
      Subset of fields from `GET /rest/api/3/issue/{id}` response. Used
      by `read`. `body_md` on IssueBody is the description rendered
      back to markdown via best-effort ADF flattening (slice 1: just
      concatenate text nodes; richer reverse-conversion is a follow-up).
    type: object
    required: [key, fields]
    properties:
      key: { type: string }
      fields:
        type: object
        required: [summary, status]
        properties:
          summary: { type: string }
          status:
            type: object
            required: [statusCategory]
            properties:
              statusCategory:
                type: object
                required: [key]
                properties:
                  key:
                    type: string
                    enum: [new, indeterminate, done]
          labels:
            type: array
            items: { type: string }
            default: []
          description:
            $ref: "#/definitions/AdfDocument"
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: jira_create_happy_path
    title: JiraIssueBackend creates an issue via REST API and returns the key
    description: >
      When create(title) is called and the mock HTTP server returns a
      201 Created with `{"key": "PROJ-42"}`, the backend extracts the
      key and returns it as IssueId. Request body must be valid ADF.
    given:
      - JIRA_BASE_URL, JIRA_USER_EMAIL, JIRA_API_TOKEN, JIRA_PROJECT_KEY
        env vars are all set
      - The mock HTTP server matches `POST /rest/api/3/issue` and
        responds 201 with `{"key": "PROJ-42", "id": "10042"}`
    when:
      - JiraIssueBackend::create("dashboard widget") is called
    then:
      - The returned IssueId equals "PROJ-42"
      - The request Authorization header contains
        `Basic base64(email:token)`
      - The request body's `fields.project.key` matches JIRA_PROJECT_KEY
      - The request body's `fields.description` is a valid ADF document
    acceptance:
      - test: projects/agentic-workflow/tests/jira_backend_tests.rs::create_happy_path
        assertion: result == Ok(IssueId::new("PROJ-42"))

  - id: jira_search_open_issues
    title: JiraIssueBackend list composes JQL from ListFilter
    description: >
      list(filter) constructs JQL `project=<KEY> AND
      statusCategory!=Done` for state=Open, mapping each issue in the
      response to IssueRef.
    given:
      - JIRA env vars set
      - Mock HTTP responds to `GET /rest/api/3/search/jql?jql=...` with
        a JSON body containing two issues with statusCategory keys
        `new` and `done`
    when:
      - JiraIssueBackend::list(&ListFilter { state: Open, labels: [] })
        is called
    then:
      - The JQL string includes `project=` and `statusCategory!=Done`
      - The result Vec<IssueRef> maps statusCategory.key to IssueState
        (new/indeterminate -> Open, done -> Closed)
    acceptance:
      - test: projects/agentic-workflow/tests/jira_backend_tests.rs::list_open_issues_jql
        assertion: refs.len() == 2 with mapped states
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/runtime/jira_backend.rs -->
```rust
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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
# JiraIssueBackend is fully regenerable through the source template. Cargo
# dependency wiring and HTTP integration tests remain hand-written.
changes:
  - path: projects/agentic-workflow/Cargo.toml
    action: modify
    section: source
    impl_mode: hand-written
    description: >
      Add `reqwest = { version = "0.12", default-features = false,
      features = ["json", "rustls-tls"] }` and `base64 = "0.22"`
      dependencies. rustls-tls avoids OpenSSL system dep on dev machines.

  - path: projects/agentic-workflow/src/runtime/jira_backend.rs
    action: modify
    section: source
    impl_mode: codegen
    description: >
      Source template for JiraIssueBackend, env-based REST configuration,
      Basic auth header construction, ADF wrapping/flattening, JQL
      construction, response mapping, and IssueBackend implementation.
      update() and close() remain Unsupported in slice 1.

  - path: projects/agentic-workflow/tests/jira_backend_tests.rs
    action: modify
    section: source
    impl_mode: hand-written
    description: >
      Optional integration test surface using a mock HTTP server (wiremock or
      hyper). It remains hand-written and outside this source-template
      promotion.
  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```
