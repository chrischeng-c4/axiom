// HANDWRITE-BEGIN gap="missing-generator:logic:c0ur1e03" tracker="pending-tracker" reason="Outbound GitHub Issues forwarding: GithubClient holds the server-held COURIER_GITHUB_TOKEN + COURIER_ALLOWED_REPOS allow-list, and forwards search/view/create/comment to api.github.com with the exact headers libs/cli-std/src/issue.rs already uses (Accept: application/vnd.github+json, bearer_auth), so courier's wire behavior matches the direct-GitHub path it replaces."
//! Outbound calls to `api.github.com` on courier's server-held credential.
//!
//! [`GithubClient`] is the one place courier talks to GitHub: it forwards
//! `search`/`view`/`create`/`comment` with the exact request shape
//! `libs/cli-std/src/issue.rs` already uses against GitHub directly (same
//! `Accept: application/vnd.github+json` header, same bearer-auth scheme),
//! so a CLI that switches from calling GitHub directly to calling courier
//! sees byte-identical upstream behavior. `comment` mirrors `issue.rs`'s
//! reopen-then-comment semantics (PATCH `state=open`, then `POST` the
//! comment).
//!
//! [`GithubClient::is_allowed`] enforces `COURIER_ALLOWED_REPOS` — courier
//! holds one shared credential, so an allow-list keeps it from being used as
//! an open relay to arbitrary repos.

use anyhow::{Context, Result};
use reqwest::StatusCode;
use serde_json::Value;

/// Env var for the real GitHub credential courier holds server-side.
/// Required at startup — a courier that can never call GitHub is a
/// misconfiguration, not a per-request failure.
pub const GITHUB_TOKEN_ENV: &str = "COURIER_GITHUB_TOKEN";
/// Env var for the comma-separated `owner/name` allow-list. Defaults to
/// `chrischeng-c4/axiom` when unset.
pub const ALLOWED_REPOS_ENV: &str = "COURIER_ALLOWED_REPOS";
const DEFAULT_ALLOWED_REPO: &str = "chrischeng-c4/axiom";

/// Why a GitHub call failed: transport/parse failure vs. a GitHub-returned
/// error status, kept distinct so `routes.rs` can map them to different
/// HTTP statuses (502 vs. the upstream status).
#[derive(Debug)]
pub enum GithubError {
    Upstream(String),
    Github { status: StatusCode, message: String },
}

/// Forwards issue operations to `api.github.com` with the server-held
/// credential.
pub struct GithubClient {
    http: reqwest::Client,
    token: String,
    allowed_repos: Vec<String>,
}

impl GithubClient {
    /// Resolve the credential + allow-list from env. Fails fast when
    /// [`GITHUB_TOKEN_ENV`] is unset — matches [`crate::http::auth::AuthConfig::resolve`]'s
    /// fail-fast-at-startup discipline.
    pub fn from_env() -> Result<Self> {
        let token = std::env::var(GITHUB_TOKEN_ENV).with_context(|| {
            format!(
                "{GITHUB_TOKEN_ENV} must be set — courier holds the real GitHub credential \
                 server-side"
            )
        })?;
        let allowed_repos = match std::env::var(ALLOWED_REPOS_ENV) {
            Ok(v) if !v.trim().is_empty() => {
                v.split(',').map(|s| s.trim().to_string()).collect()
            }
            _ => vec![DEFAULT_ALLOWED_REPO.to_string()],
        };
        let http = reqwest::Client::builder()
            .user_agent(format!("courier/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .context("build GitHub HTTP client")?;
        Ok(Self {
            http,
            token,
            allowed_repos,
        })
    }

    /// Whether `owner/name` is in [`ALLOWED_REPOS_ENV`] (or its default).
    pub fn is_allowed(&self, owner: &str, name: &str) -> bool {
        let repo = format!("{owner}/{name}");
        self.allowed_repos.iter().any(|r| r == &repo)
    }

    /// `GET /search/issues` scoped to one repo — mirrors
    /// `issue.rs::search`'s query shape (`repo:` + `is:issue` + optional
    /// `state:`/free text).
    pub async fn search_issues(
        &self,
        owner: &str,
        name: &str,
        state: &str,
        q: Option<&str>,
        limit: u32,
    ) -> Result<Value, GithubError> {
        let mut query = format!("repo:{owner}/{name} is:issue");
        if state != "all" {
            query.push_str(&format!(" state:{state}"));
        }
        if let Some(text) = q.filter(|t| !t.trim().is_empty()) {
            query.push(' ');
            query.push_str(text.trim());
        }
        let url = format!(
            "https://api.github.com/search/issues?q={}&per_page={limit}",
            percent_encode_query(&query),
        );
        self.get(&url).await
    }

    /// `GET /repos/{owner}/{name}/issues/{number}`.
    pub async fn view_issue(&self, owner: &str, name: &str, number: u64) -> Result<Value, GithubError> {
        let url = format!("https://api.github.com/repos/{owner}/{name}/issues/{number}");
        self.get(&url).await
    }

    /// `POST /repos/{owner}/{name}/issues` — `payload` is forwarded
    /// verbatim (the caller shapes `title`/`body`/`labels`, same as
    /// `issue.rs::issue_payload`).
    pub async fn create_issue(
        &self,
        owner: &str,
        name: &str,
        payload: &Value,
    ) -> Result<Value, GithubError> {
        let url = format!("https://api.github.com/repos/{owner}/{name}/issues");
        self.send(self.http.post(&url).json(payload)).await
    }

    /// Reopen (`PATCH state=open`, idempotent when already open), then
    /// `POST` the comment — matches `issue.rs::comment`'s
    /// reopen-then-comment semantics.
    pub async fn comment_issue(
        &self,
        owner: &str,
        name: &str,
        number: u64,
        payload: &Value,
    ) -> Result<Value, GithubError> {
        let issue_url = format!("https://api.github.com/repos/{owner}/{name}/issues/{number}");
        let mut reopen = serde_json::Map::new();
        reopen.insert("state".into(), "open".into());
        self.send(self.http.patch(&issue_url).json(&Value::Object(reopen)))
            .await?;

        let comments_url = format!("{issue_url}/comments");
        self.send(self.http.post(&comments_url).json(payload)).await
    }

    async fn get(&self, url: &str) -> Result<Value, GithubError> {
        self.send(self.http.get(url)).await
    }

    async fn send(&self, builder: reqwest::RequestBuilder) -> Result<Value, GithubError> {
        let resp = builder
            .header("Accept", "application/vnd.github+json")
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| GithubError::Upstream(e.to_string()))?;
        let status = resp.status();
        let value: Value = resp
            .json()
            .await
            .map_err(|e| GithubError::Upstream(format!("parse GitHub response: {e}")))?;
        if !status.is_success() {
            let message = value
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error")
                .to_string();
            return Err(GithubError::Github { status, message });
        }
        Ok(value)
    }
}

/// Percent-encode a GitHub search query string (mirrors
/// `libs/cli-std/src/issue.rs`'s local encoder — no new dependency for one
/// query param).
fn percent_encode_query(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_list_matches_owner_slash_name() {
        let client = GithubClient {
            http: reqwest::Client::new(),
            token: "t".into(),
            allowed_repos: vec!["chrischeng-c4/axiom".into()],
        };
        assert!(client.is_allowed("chrischeng-c4", "axiom"));
        assert!(!client.is_allowed("someone-else", "other-repo"));
    }

    #[test]
    fn percent_encode_query_escapes_reserved_bytes() {
        assert_eq!(
            percent_encode_query("repo:a/b is:issue state:open"),
            "repo%3Aa%2Fb%20is%3Aissue%20state%3Aopen"
        );
    }
}
// HANDWRITE-END
