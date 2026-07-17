//! Axiom Issue Proxy Client
//!
//! Handles direct GitHub or Courier proxy routing automatically based on
//! `AXIOM_COURIER_URL` / `GITHUB_TOKEN`.

use anyhow::{bail, Context, Result};
use cli_std::issue::IssueBackend;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Client as HttpClient;
use serde_json::json;

/// Percent-encode query parameters exactly as `cli_std` did.
fn percent_encode_query(s: &str) -> String {
    utf8_percent_encode(s, NON_ALPHANUMERIC).to_string()
}

pub struct Client {
    http: HttpClient,
}

impl Client {
    pub fn new(project: &str, version: &str) -> Result<Self> {
        let http = HttpClient::builder()
            .user_agent(format!("{project}-issue/{version}"))
            .build()
            .context("build HTTP client")?;
        Ok(Self { http })
    }

    /// Resolve the Courier URL from environment, if any.
    fn resolve_courier_url() -> Option<String> {
        std::env::var("AXIOM_COURIER_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
    }

    /// Resolve the Courier Token from environment.
    fn resolve_courier_token() -> Option<String> {
        std::env::var("AXIOM_COURIER_TOKEN")
            .ok()
            .filter(|s| !s.trim().is_empty())
    }

    /// Resolve the GitHub Token from environment.
    fn resolve_github_token() -> Option<String> {
        std::env::var("GITHUB_TOKEN")
            .ok()
            .filter(|s| !s.trim().is_empty())
    }

    // -- COURIER HELPERS --

    fn courier_search_url(courier_url: &str, repo: &str, state: &str, q: &str, limit: u32) -> String {
        format!(
            "{}/issues?repo={}&state={}&q={}&limit={limit}",
            courier_url.trim_end_matches('/'),
            percent_encode_query(repo),
            percent_encode_query(state),
            percent_encode_query(q),
        )
    }

    fn courier_view_url(courier_url: &str, repo: &str, number: u64) -> String {
        format!(
            "{}/issues?repo={}&number={}",
            courier_url.trim_end_matches('/'),
            percent_encode_query(repo),
            number
        )
    }

    async fn courier_get(&self, url: &str) -> Result<reqwest::Response> {
        let mut req = self.http.get(url);
        if let Some(token) = Self::resolve_courier_token() {
            req = req.bearer_auth(token);
        }
        req.send()
            .await
            .with_context(|| format!("GET {url}"))?
            .error_for_status()
            .with_context(|| format!("courier error for {url}"))
    }

    async fn courier_post(&self, url: &str, payload: &serde_json::Value) -> Result<reqwest::Response> {
        let mut req = self.http.post(url).json(payload);
        if let Some(token) = Self::resolve_courier_token() {
            req = req.bearer_auth(token);
        }
        req.send()
            .await
            .with_context(|| format!("POST {url}"))?
            .error_for_status()
            .with_context(|| format!("courier error for {url}"))
    }

    // -- DIRECT GITHUB HELPERS --

    fn github_search_url(q: &str, limit: u32) -> String {
        format!(
            "https://api.github.com/search/issues?q={}&per_page={limit}",
            percent_encode_query(q)
        )
    }

    fn github_view_url(repo: &str, number: u64) -> String {
        format!("https://api.github.com/repos/{repo}/issues/{number}")
    }

    async fn github_get(&self, url: &str) -> Result<reqwest::Response> {
        let mut req = self.http.get(url).header("Accept", "application/vnd.github+json");
        if let Some(token) = Self::resolve_github_token() {
            req = req.bearer_auth(token);
        }
        req.send()
            .await
            .with_context(|| format!("GET {url}"))?
            .error_for_status()
            .with_context(|| format!("GitHub error for {url}"))
    }
}

#[async_trait::async_trait]
impl IssueBackend for Client {
    async fn search(&self, repo: &str, state: &str, query: &str, limit: u32) -> Result<serde_json::Value> {
        let q = if query.is_empty() {
            format!("repo:{repo} is:issue label:\"app:{}\"", repo.split('/').last().unwrap_or(""))
        } else {
            format!("repo:{repo} is:issue {query}")
        };

        if let Some(courier_url) = Self::resolve_courier_url() {
            let url = Self::courier_search_url(&courier_url, repo, state, &q, limit);
            self.courier_get(&url).await?.json().await.context("parse courier issue search response")
        } else {
            let url = Self::github_search_url(&q, limit);
            let val: serde_json::Value = self.github_get(&url).await?.json().await.context("parse github search")?;
            Ok(val.get("items").cloned().unwrap_or(json!([])))
        }
    }

    async fn view(&self, repo: &str, number: u64) -> Result<serde_json::Value> {
        if let Some(courier_url) = Self::resolve_courier_url() {
            let url = Self::courier_view_url(&courier_url, repo, number);
            self.courier_get(&url).await?.json().await.context("parse courier issue response")
        } else {
            let url = Self::github_view_url(repo, number);
            self.github_get(&url).await?.json().await.context("parse github issue response")
        }
    }

    async fn submit_issue(&self, repo: &str, payload: &serde_json::Value) -> Result<String> {
        if let Some(courier_url) = Self::resolve_courier_url() {
            let url = format!("{}/issues", courier_url.trim_end_matches('/'));
            let batch_payload = json!({
                "ops": [
                    { "method": "create", "repo": repo, "payload": payload }
                ]
            });
            let results: serde_json::Value = self.courier_post(&url, &batch_payload).await?.json().await?;
            let status = results.get(0).and_then(|r| r.get("status").and_then(|s| s.as_u64())).unwrap_or(500);
            if status >= 400 {
                let err_msg = results.get(0).and_then(|r| r.get("body").and_then(|b| b.as_str())).unwrap_or("unknown error");
                bail!("courier batch op failed (status {status}): {err_msg}");
            }
            let body = results.get(0).and_then(|r| r.get("body")).context("empty batch response")?;
            Ok(body.get("html_url").and_then(|u| u.as_str()).unwrap_or("(issue created)").to_string())
        } else {
            let url = format!("https://api.github.com/repos/{repo}/issues");
            let token = Self::resolve_github_token().context("no GITHUB_TOKEN available")?;
            let resp = self.http.post(&url).header("Accept", "application/vnd.github+json").bearer_auth(token).json(payload).send().await.context("POST issue")?;
            let status = resp.status();
            let val: serde_json::Value = resp.json().await?;
            if !status.is_success() {
                let msg = val.get("message").and_then(|m| m.as_str()).unwrap_or("unknown error");
                bail!("GitHub returned {status}: {msg}");
            }
            Ok(val.get("html_url").and_then(|u| u.as_str()).unwrap_or("(issue created)").to_string())
        }
    }

    async fn reopen_issue(&self, repo: &str, number: u64) -> Result<String> {
        if let Some(courier_url) = Self::resolve_courier_url() {
            let url = format!("{}/issues", courier_url.trim_end_matches('/'));
            let batch_payload = json!({
                "ops": [
                    { "method": "update", "repo": repo, "number": number, "payload": { "state": "open" } }
                ]
            });
            let results: serde_json::Value = self.courier_post(&url, &batch_payload).await?.json().await?;
            let status = results.get(0).and_then(|r| r.get("status").and_then(|s| s.as_u64())).unwrap_or(500);
            if status >= 400 {
                let err_msg = results.get(0).and_then(|r| r.get("body").and_then(|b| b.as_str())).unwrap_or("unknown error");
                bail!("courier batch op failed (status {status}): {err_msg}");
            }
            let body = results.get(0).and_then(|r| r.get("body")).context("empty batch response")?;
            Ok(body.get("html_url").and_then(|u| u.as_str()).unwrap_or("(issue reopened)").to_string())
        } else {
            let url = format!("https://api.github.com/repos/{repo}/issues/{number}");
            let token = Self::resolve_github_token().context("no GITHUB_TOKEN available")?;
            let payload = json!({"state": "open"});
            let resp = self.http.patch(&url).header("Accept", "application/vnd.github+json").bearer_auth(token).json(&payload).send().await.context("PATCH issue")?;
            let status = resp.status();
            let val: serde_json::Value = resp.json().await?;
            if !status.is_success() {
                let msg = val.get("message").and_then(|m| m.as_str()).unwrap_or("unknown error");
                bail!("GitHub returned {status}: {msg}");
            }
            Ok(val.get("html_url").and_then(|u| u.as_str()).unwrap_or("(issue reopened)").to_string())
        }
    }

    async fn post_issue_comment(&self, repo: &str, number: u64, body: &str) -> Result<String> {
        if let Some(courier_url) = Self::resolve_courier_url() {
            let url = format!("{}/issues", courier_url.trim_end_matches('/'));
            let batch_payload = json!({
                "ops": [
                    { "method": "comment", "repo": repo, "number": number, "payload": { "body": body } }
                ]
            });
            let results: serde_json::Value = self.courier_post(&url, &batch_payload).await?.json().await?;
            let status = results.get(0).and_then(|r| r.get("status").and_then(|s| s.as_u64())).unwrap_or(500);
            if status >= 400 {
                let err_msg = results.get(0).and_then(|r| r.get("body").and_then(|b| b.as_str())).unwrap_or("unknown error");
                bail!("courier batch op failed (status {status}): {err_msg}");
            }
            let res_body = results.get(0).and_then(|r| r.get("body")).context("empty batch response")?;
            Ok(res_body.get("html_url").and_then(|u| u.as_str()).unwrap_or("(comment created)").to_string())
        } else {
            let url = format!("https://api.github.com/repos/{repo}/issues/{number}/comments");
            let token = Self::resolve_github_token().context("no GITHUB_TOKEN available")?;
            let payload = json!({"body": body});
            let resp = self.http.post(&url).header("Accept", "application/vnd.github+json").bearer_auth(token).json(&payload).send().await.context("POST issue comment")?;
            let status = resp.status();
            let val: serde_json::Value = resp.json().await?;
            if !status.is_success() {
                let msg = val.get("message").and_then(|m| m.as_str()).unwrap_or("unknown error");
                bail!("GitHub returned {status}: {msg}");
            }
            Ok(val.get("html_url").and_then(|u| u.as_str()).unwrap_or("(comment created)").to_string())
        }
    }

    async fn fetch_node_status(&self, url: &str) -> String {
        let fetch = async {
            let version = self.http
                .get(format!("{}/version", url.trim_end_matches('/')))
                .send()
                .await?
                .text()
                .await?;
            let health = self.http
                .get(format!("{}/healthz", url.trim_end_matches('/')))
                .send()
                .await?
                .text()
                .await?;
            Ok::<String, anyhow::Error>(format!(
                "node: {url}\nversion: {version}\nhealthz: {health}"
            ))
        };
        match fetch.await {
            Ok(s) => s,
            Err(e) => format!("node: {url}\nstatus-error: {e}"),
        }
    }

    fn has_credential(&self) -> bool {
        Self::resolve_courier_token().is_some() || Self::resolve_github_token().is_some()
    }
}
