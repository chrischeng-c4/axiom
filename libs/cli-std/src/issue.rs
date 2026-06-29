//! `<tool> issue <verb>` — the shared issue interface every CLI ships.
//!
//! - [`search`] — find this tool's issues on the tracker (filtered to the
//!   `project:<name>` label), optionally by free text. Read-only.
//! - [`view`] — print a single issue by number. Read-only.
//! - [`create`] — assemble a diagnostics block + the operator's description and
//!   file a GitHub issue (`POST /repos/{repo}/issues` via `GITHUB_TOKEN`), or
//!   print a pre-filled `issues/new` URL when no token is available.
//!   `--dry-run` prints without submitting.
//!
//! Body assembly / URL pre-fill / repo resolution / payload shaping are pure and
//! unit-tested; everything network-facing lives behind the `online` feature.

use crate::ToolInfo;
use anyhow::Result;

// ---------------------------------------------------------------------------
// create
// ---------------------------------------------------------------------------

/// Flags for `issue create`.
#[derive(Clone, Debug, Default)]
pub struct CreateOptions {
    pub title: String,
    pub message: Option<String>,
    /// Optional running node to enrich the report from (`/version`+`/healthz`).
    pub url: Option<String>,
    /// Override the target repo (`owner/name`); defaults to `tool.repo`.
    pub repo: Option<String>,
    pub label: Vec<String>,
    pub dry_run: bool,
    pub yes: bool,
}

/// Render the diagnostics block from the tool identity (+ optional node line).
pub fn render_diagnostics(tool: &ToolInfo, node: Option<&str>) -> String {
    let mut s = String::from("## Diagnostics\n");
    s.push_str(&format!("- {} version: {}\n", tool.project, tool.version));
    s.push_str(&format!("- target: {}\n", tool.target));
    s.push_str(&format!("- git sha: {}\n", tool.git_sha));
    s.push_str(&format!("- built at: {}\n", tool.built_at));
    s.push_str(&format!(
        "- os/arch: {}/{}\n",
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
    if let Some(node) = node {
        s.push_str(&format!("- node: {node}\n"));
    }
    s
}

/// Assemble the issue body: message first (when non-empty), separator, then the
/// diagnostics block.
pub fn assemble_body(message: Option<&str>, diagnostics: &str) -> String {
    match message {
        Some(m) if !m.trim().is_empty() => format!("{}\n\n---\n{diagnostics}", m.trim()),
        _ => diagnostics.to_string(),
    }
}

/// The repo to file against: `--repo` else the tool's default.
pub fn resolve_repo<'a>(tool: &'a ToolInfo, repo: Option<&'a str>) -> &'a str {
    repo.unwrap_or(tool.repo)
}

/// The GitHub issue-creation JSON payload (`labels` omitted when empty).
pub fn issue_payload(title: &str, body: &str, labels: &[String]) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("title".into(), title.into());
    map.insert("body".into(), body.into());
    if !labels.is_empty() {
        map.insert("labels".into(), labels.iter().cloned().collect());
    }
    serde_json::Value::Object(map)
}

/// A browser-openable pre-filled `issues/new` URL (title + body + labels
/// percent-encoded). Labels are comma-joined into the `labels` query param so
/// the convention's `project:<name>` tag survives the no-token fallback path.
pub fn prefilled_url(repo: &str, title: &str, body: &str, labels: &[String]) -> String {
    let mut url = format!(
        "https://github.com/{repo}/issues/new?title={}&body={}",
        percent_encode_query(title),
        percent_encode_query(body),
    );
    if !labels.is_empty() {
        url.push_str(&format!("&labels={}", percent_encode_query(&labels.join(","))));
    }
    url
}

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

fn print_preview(repo: &str, title: &str, body: &str, labels: &[String]) {
    println!("repo:  {repo}");
    println!("title: {title}");
    if !labels.is_empty() {
        println!("labels: {}", labels.join(", "));
    }
    println!("---");
    println!("{body}");
}

/// Print the pre-filled-issue URL plus the title/body so the user can file by
/// hand. The preceding diagnostic note (why we fell back) is the caller's
/// responsibility — the "no credential" and "offline build" conditions are
/// distinct and must not be conflated.
fn print_fallback(repo: &str, title: &str, body: &str, labels: &[String]) {
    println!("{}", prefilled_url(repo, title, body, labels));
    eprintln!("\n--- title ---\n{title}\n--- body ---\n{body}");
}

/// Online build, but no GitHub credential was found anywhere.
#[cfg(feature = "online")]
fn note_no_credential() {
    eprintln!(
        "note: no GitHub credential found (checked $GH_TOKEN, $GITHUB_TOKEN, and `gh auth token`). \
         Run `gh auth login` or set GITHUB_TOKEN to file directly. \
         Meanwhile, open this pre-filled issue:"
    );
}

/// This binary was built without the `online` feature, so it cannot do network
/// I/O at all — independent of whether a credential exists.
#[cfg(not(feature = "online"))]
fn note_offline_build() {
    eprintln!(
        "note: this jet build has no `online` feature; it cannot file directly. \
         Open this pre-filled issue:"
    );
}

/// `issue create` — file (or preview) a structured issue.
#[cfg(feature = "online")]
pub async fn create(tool: &ToolInfo, opts: CreateOptions) -> Result<()> {
    let repo = resolve_repo(tool, opts.repo.as_deref()).to_string();
    let client = http_client(tool)?;

    let node = match opts.url.as_deref() {
        Some(url) => Some(fetch_node_status(&client, url).await),
        None => None,
    };
    let body = assemble_body(
        opts.message.as_deref(),
        &render_diagnostics(tool, node.as_deref()),
    );

    if opts.dry_run {
        print_preview(&repo, &opts.title, &body, &opts.label);
        return Ok(());
    }

    match crate::resolve_github_token() {
        Some(token) => {
            if !opts.yes && !crate::confirm(&format!("file this issue to {repo}?"))? {
                println!("aborted");
                return Ok(());
            }
            let url = submit_issue(
                &client,
                &repo,
                &token,
                &issue_payload(&opts.title, &body, &opts.label),
            )
            .await?;
            println!("filed: {url}");
        }
        None => {
            note_no_credential();
            print_fallback(&repo, &opts.title, &body, &opts.label);
        }
    }
    Ok(())
}

/// Offline build: assemble + print (`--dry-run`) or the browser fallback.
#[cfg(not(feature = "online"))]
pub async fn create(tool: &ToolInfo, opts: CreateOptions) -> Result<()> {
    let repo = resolve_repo(tool, opts.repo.as_deref()).to_string();
    let body = assemble_body(opts.message.as_deref(), &render_diagnostics(tool, None));
    if opts.dry_run {
        print_preview(&repo, &opts.title, &body, &opts.label);
    } else {
        note_offline_build();
        print_fallback(&repo, &opts.title, &body, &opts.label);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// search (read)
// ---------------------------------------------------------------------------

/// Flags for `issue search`.
#[derive(Clone, Debug)]
pub struct SearchOptions {
    /// Free-text query; `None`/empty lists recent issues for this tool.
    pub query: Option<String>,
    /// `open` (default), `closed`, or `all`.
    pub state: String,
    /// Max results.
    pub limit: u32,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            query: None,
            state: "open".to_string(),
            limit: 20,
        }
    }
}

/// `issue search` — list/search this tool's issues (filtered to `project:<name>`).
#[cfg(feature = "online")]
pub async fn search(tool: &ToolInfo, opts: SearchOptions) -> Result<()> {
    use anyhow::Context;
    let label = format!("project:{}", tool.project);
    let mut q = format!("repo:{} is:issue label:\"{}\"", tool.repo, label);
    if opts.state != "all" {
        q.push_str(&format!(" state:{}", opts.state));
    }
    if let Some(text) = opts.query.as_deref() {
        if !text.trim().is_empty() {
            q.push(' ');
            q.push_str(text.trim());
        }
    }
    let url = format!(
        "https://api.github.com/search/issues?q={}&per_page={}",
        percent_encode_query(&q),
        opts.limit
    );
    let client = http_client(tool)?;
    let v: serde_json::Value = crate::github_get(&client, &url)
        .await?
        .json()
        .await
        .context("parse issue search response")?;

    let items = v.get("items").and_then(|i| i.as_array());
    match items {
        Some(items) if !items.is_empty() => {
            for it in items {
                let num = it.get("number").and_then(|n| n.as_u64()).unwrap_or(0);
                let state = it.get("state").and_then(|s| s.as_str()).unwrap_or("?");
                let title = it.get("title").and_then(|t| t.as_str()).unwrap_or("");
                println!("#{num} [{state}] {title}");
            }
        }
        _ => println!("no {label} issues match"),
    }
    Ok(())
}

#[cfg(not(feature = "online"))]
pub async fn search(_tool: &ToolInfo, _opts: SearchOptions) -> Result<()> {
    anyhow::bail!("this build has no `online` feature — `issue search` needs network access")
}

// ---------------------------------------------------------------------------
// view (read)
// ---------------------------------------------------------------------------

/// `issue view` — print a single issue by number.
#[cfg(feature = "online")]
pub async fn view(tool: &ToolInfo, number: u64) -> Result<()> {
    use anyhow::Context;
    let url = format!("https://api.github.com/repos/{}/issues/{}", tool.repo, number);
    let client = http_client(tool)?;
    let v: serde_json::Value = crate::github_get(&client, &url)
        .await?
        .json()
        .await
        .context("parse issue response")?;

    let state = v.get("state").and_then(|s| s.as_str()).unwrap_or("?");
    let title = v.get("title").and_then(|t| t.as_str()).unwrap_or("");
    let html = v.get("html_url").and_then(|u| u.as_str()).unwrap_or("");
    let body = v.get("body").and_then(|b| b.as_str()).unwrap_or("");
    println!("#{number} [{state}] {title}");
    if !html.is_empty() {
        println!("{html}");
    }
    println!("---");
    println!("{}", if body.trim().is_empty() { "(no description)" } else { body });
    Ok(())
}

#[cfg(not(feature = "online"))]
pub async fn view(_tool: &ToolInfo, _number: u64) -> Result<()> {
    anyhow::bail!("this build has no `online` feature — `issue view` needs network access")
}

// ---------------------------------------------------------------------------
// shared online helpers
// ---------------------------------------------------------------------------

#[cfg(feature = "online")]
fn http_client(tool: &ToolInfo) -> Result<reqwest::Client> {
    use anyhow::Context;
    reqwest::Client::builder()
        .user_agent(format!("{}-issue/{}", tool.project, tool.version))
        .build()
        .context("build HTTP client")
}

#[cfg(feature = "online")]
async fn fetch_node_status(client: &reqwest::Client, url: &str) -> String {
    let base = url.trim_end_matches('/');
    match client
        .get(format!("{base}/version"))
        .send()
        .await
        .and_then(|r| r.error_for_status())
    {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            let health = client
                .get(format!("{base}/healthz"))
                .send()
                .await
                .map(|r| r.status().as_u16().to_string())
                .unwrap_or_else(|_| "?".to_string());
            format!("{base} → version={} healthz={health}", body.trim())
        }
        Err(_) => format!("unreachable ({base})"),
    }
}

#[cfg(feature = "online")]
async fn submit_issue(
    client: &reqwest::Client,
    repo: &str,
    token: &str,
    payload: &serde_json::Value,
) -> Result<String> {
    use anyhow::{bail, Context};
    let url = format!("https://api.github.com/repos/{repo}/issues");
    let resp = client
        .post(&url)
        .header("Accept", "application/vnd.github+json")
        .bearer_auth(token)
        .json(payload)
        .send()
        .await
        .context("POST issue")?;
    let status = resp.status();
    let value: serde_json::Value = resp.json().await.context("parse issue response")?;
    if !status.is_success() {
        let msg = value
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error");
        bail!("GitHub returned {status}: {msg}");
    }
    Ok(value
        .get("html_url")
        .and_then(|u| u.as_str())
        .unwrap_or("(issue created)")
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOOL: ToolInfo = ToolInfo {
        project: "lumen",
        repo: "chrischeng-c4/axiom",
        target: "aarch64-apple-darwin",
        version: "0.4.3",
        git_sha: "abc1234",
        built_at: "1700000000",
    };

    #[test]
    fn diagnostics_and_body() {
        let d = render_diagnostics(&TOOL, None);
        for n in ["lumen version: 0.4.3", "aarch64-apple-darwin", "abc1234"] {
            assert!(d.contains(n), "missing {n}");
        }
        let b = assemble_body(Some("boom"), &d);
        assert!(b.find("boom").unwrap() < b.find("## Diagnostics").unwrap());
        assert!(assemble_body(None, &d).starts_with("## Diagnostics"));
    }

    #[test]
    fn url_and_repo_and_payload() {
        let u = prefilled_url("o/n", "a b&c", "x\ny", &[]);
        assert!(u.starts_with("https://github.com/o/n/issues/new?title="));
        assert!(u.contains("a%20b%26c") && u.contains("x%0Ay") && !u.contains(' '));
        assert!(!u.contains("labels="));
        // Labels survive the no-token URL fallback (convention `project:<name>`).
        let ul = prefilled_url("o/n", "t", "b", &["project:jet".into(), "bug".into()]);
        assert!(ul.contains("&labels=project%3Ajet%2Cbug"));
        assert_eq!(resolve_repo(&TOOL, None), "chrischeng-c4/axiom");
        assert_eq!(resolve_repo(&TOOL, Some("o/n")), "o/n");

        let p = issue_payload("t", "b", &["bug".into()]);
        assert_eq!(p["title"], "t");
        assert_eq!(p["labels"], serde_json::json!(["bug"]));
        assert!(issue_payload("t", "b", &[]).get("labels").is_none());
    }
}
