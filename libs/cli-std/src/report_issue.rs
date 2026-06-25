//! `<tool> report-issue` — assemble a diagnostics block + the operator's
//! description and file a GitHub issue (`POST /repos/{repo}/issues` via
//! `GITHUB_TOKEN`), or print a pre-filled `issues/new` URL when no token is
//! available. `--dry-run` prints without submitting.
//!
//! Body assembly / URL pre-fill / repo resolution / payload shaping are pure
//! and unit-tested; the node-probe + submit path lives behind `online`.

use crate::ToolInfo;
use anyhow::Result;

/// Flags for a report-issue run.
#[derive(Clone, Debug, Default)]
pub struct Options {
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

fn print_fallback(repo: &str, title: &str, body: &str, labels: &[String]) {
    eprintln!(
        "note: no GITHUB_TOKEN set (or built without the `online` feature) — \
         open this pre-filled issue, or set GITHUB_TOKEN to file it directly:"
    );
    println!("{}", prefilled_url(repo, title, body, labels));
    eprintln!("\n--- title ---\n{title}\n--- body ---\n{body}");
}

/// Run `<tool> report-issue`.
#[cfg(feature = "online")]
pub async fn run(tool: &ToolInfo, opts: Options) -> Result<()> {
    use anyhow::Context;

    let repo = resolve_repo(tool, opts.repo.as_deref()).to_string();
    let client = reqwest::Client::builder()
        .user_agent(format!("{}-report-issue/{}", tool.project, tool.version))
        .build()
        .context("build HTTP client")?;

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

    match std::env::var("GITHUB_TOKEN").ok().filter(|t| !t.is_empty()) {
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
        None => print_fallback(&repo, &opts.title, &body, &opts.label),
    }
    Ok(())
}

/// Offline build: assemble + print (`--dry-run`) or the browser fallback.
#[cfg(not(feature = "online"))]
pub async fn run(tool: &ToolInfo, opts: Options) -> Result<()> {
    let repo = resolve_repo(tool, opts.repo.as_deref()).to_string();
    let body = assemble_body(opts.message.as_deref(), &render_diagnostics(tool, None));
    if opts.dry_run {
        print_preview(&repo, &opts.title, &body, &opts.label);
    } else {
        print_fallback(&repo, &opts.title, &body, &opts.label);
    }
    Ok(())
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
