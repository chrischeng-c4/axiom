// HANDWRITE-BEGIN gap="missing-generator:source:9b4318d9" tracker="pending-tracker" reason="Port lumen's report-issue implementation (diagnostics, body assemble, gh issue create / prefilled URL fallback) to vat."
//! `vat report-issue` — turn a problem into a diagnostics-rich GitHub issue
//! without leaving the binary.
//!
//! Assembles a diagnostics block (version, target, git sha, built-at, OS/arch)
//! together with the caller's description, then files a GitHub issue via the
//! REST API using `GITHUB_TOKEN`. Without a token (or when built without the
//! `report-issue` feature) it prints a pre-filled `issues/new` URL plus the
//! body — never a silent failure. `--dry-run` assembles and prints without
//! submitting.
//!
//! Body assembly, URL pre-fill, repo resolution and payload shaping are pure and
//! unit-tested; only the HTTP issue-submit path is gated behind the
//! `report-issue` feature (it needs the otherwise-optional HTTP client + tokio).
//!
//! @spec projects/vat/tech-design/interfaces/cli/vat-upgrade-and-report-issue-subcommands-for-the-mandatory-cli-c.md#cli

use std::process::ExitCode;

use anyhow::Result;

/// Options for a report-issue run (mirrors the CLI flags).
pub struct Options {
    /// Issue title.
    pub title: String,
    /// Free-text description placed above the diagnostics block.
    pub message: Option<String>,
    /// Target repository (`owner/name`); defaults to vat's release repo.
    pub repo: Option<String>,
    /// Labels to attach to the issue.
    pub label: Vec<String>,
    /// Assemble and print without submitting anything.
    pub dry_run: bool,
    /// Skip the confirmation prompt.
    pub yes: bool,
}

/// Build-time + runtime provenance included in every report.
pub struct Diagnostics {
    pub version: &'static str,
    pub target: &'static str,
    pub git_sha: &'static str,
    pub built_at: &'static str,
    pub os: &'static str,
    pub arch: &'static str,
}

/// Collect diagnostics from the build-time stamps (`build.rs`) and runtime
/// constants.
pub fn current_diagnostics() -> Diagnostics {
    Diagnostics {
        version: env!("CARGO_PKG_VERSION"),
        target: env!("VAT_TARGET"),
        git_sha: env!("VAT_GIT_SHA"),
        built_at: env!("VAT_BUILT_AT"),
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
    }
}

/// Render the diagnostics block as Markdown.
pub fn render_diagnostics(d: &Diagnostics) -> String {
    let mut s = String::from("## Diagnostics\n");
    s.push_str(&format!("- vat version: {}\n", d.version));
    s.push_str(&format!("- target: {}\n", d.target));
    s.push_str(&format!("- git sha: {}\n", d.git_sha));
    s.push_str(&format!("- built at (unix): {}\n", d.built_at));
    s.push_str(&format!("- os/arch: {}/{}\n", d.os, d.arch));
    s
}

/// Assemble the issue body: the caller's message first (when non-empty), then a
/// separator, then the diagnostics block.
pub fn assemble_body(message: Option<&str>, diag: &Diagnostics) -> String {
    let rendered = render_diagnostics(diag);
    match message {
        Some(m) if !m.trim().is_empty() => format!("{}\n\n---\n{rendered}", m.trim()),
        _ => rendered,
    }
}

/// The repository to file against: `--repo` when given, else vat's release repo.
pub fn resolve_repo(repo: Option<&str>) -> &str {
    repo.unwrap_or(crate::commands::upgrade::DEFAULT_REPO)
}

/// The GitHub issue-creation JSON payload. `labels` is omitted when empty.
pub fn issue_payload(title: &str, body: &str, labels: &[String]) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("title".into(), title.into());
    map.insert("body".into(), body.into());
    if !labels.is_empty() {
        map.insert("labels".into(), labels.iter().cloned().collect());
    }
    serde_json::Value::Object(map)
}

/// A browser-openable pre-filled `issues/new` URL with the title and body
/// percent-encoded into the query string.
pub fn prefilled_url(repo: &str, title: &str, body: &str) -> String {
    format!(
        "https://github.com/{repo}/issues/new?title={}&body={}",
        percent_encode_query(title),
        percent_encode_query(body),
    )
}

/// Percent-encode a query-string component per RFC 3986 (everything outside the
/// unreserved set is escaped, so spaces, newlines and `&` are safe).
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

/// Print the assembled report without submitting (the `--dry-run` path).
fn print_preview(repo: &str, title: &str, body: &str) {
    println!("repo:  {repo}");
    println!("title: {title}");
    println!("---");
    println!("{body}");
}

/// Print the browser fallback: a pre-filled `issues/new` URL on stdout, with the
/// explanatory notes and body on stderr so the URL stays pipeable.
fn print_fallback(repo: &str, title: &str, body: &str) {
    eprintln!(
        "note: no GITHUB_TOKEN set (or built without the `report-issue` feature) — \
         open this pre-filled issue, or set GITHUB_TOKEN to file it directly:"
    );
    println!("{}", prefilled_url(repo, title, body));
    eprintln!("\n--- title ---\n{title}\n--- body ---\n{body}");
}

/// Sync CLI entry. The dry-run and browser-fallback paths are pure; only an
/// authenticated submit needs the HTTP client + a tokio runtime.
pub fn exec(opts: Options) -> Result<ExitCode> {
    let repo = resolve_repo(opts.repo.as_deref()).to_string();
    let diag = current_diagnostics();
    let body = assemble_body(opts.message.as_deref(), &diag);

    if opts.dry_run {
        print_preview(&repo, &opts.title, &body);
        return Ok(ExitCode::SUCCESS);
    }

    submit_or_fallback(&repo, &opts, &body)?;
    Ok(ExitCode::SUCCESS)
}

/// With the `report-issue` feature: file via the GitHub API when `GITHUB_TOKEN`
/// is set, otherwise print the browser fallback.
#[cfg(feature = "report-issue")]
fn submit_or_fallback(repo: &str, opts: &Options, body: &str) -> Result<()> {
    match std::env::var("GITHUB_TOKEN").ok().filter(|t| !t.is_empty()) {
        Some(token) => {
            if !opts.yes && !confirm(repo)? {
                println!("aborted");
                return Ok(());
            }
            let payload = issue_payload(&opts.title, body, &opts.label);
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;
            let url = runtime.block_on(submit_issue(repo, &token, &payload))?;
            println!("filed: {url}");
        }
        None => print_fallback(repo, &opts.title, body),
    }
    Ok(())
}

/// Without the `report-issue` feature the HTTP client is not linked, so a
/// non-dry-run always degrades to the browser fallback. It never silently fails.
#[cfg(not(feature = "report-issue"))]
fn submit_or_fallback(repo: &str, opts: &Options, body: &str) -> Result<()> {
    print_fallback(repo, &opts.title, body);
    Ok(())
}

/// POST the issue and return its `html_url`.
#[cfg(feature = "report-issue")]
async fn submit_issue(repo: &str, token: &str, payload: &serde_json::Value) -> Result<String> {
    use anyhow::{bail, Context};

    let client = reqwest::Client::builder()
        .user_agent(concat!("vat/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("build HTTP client")?;
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

#[cfg(feature = "report-issue")]
fn confirm(repo: &str) -> Result<bool> {
    use anyhow::Context;
    use std::io::{IsTerminal, Write};
    if !std::io::stdin().is_terminal() {
        return Ok(true);
    }
    print!("file this issue to {repo}? [y/N] ");
    std::io::stdout().flush().ok();
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("read confirmation")?;
    Ok(matches!(line.trim(), "y" | "Y" | "yes" | "Yes"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diag() -> Diagnostics {
        Diagnostics {
            version: "0.3.62",
            target: "aarch64-apple-darwin",
            git_sha: "abc1234",
            built_at: "1700000000",
            os: "macos",
            arch: "aarch64",
        }
    }

    #[test]
    fn render_diagnostics_has_all_fields() {
        let r = render_diagnostics(&diag());
        for needle in [
            "0.3.62",
            "aarch64-apple-darwin",
            "abc1234",
            "1700000000",
            "macos",
            "aarch64",
        ] {
            assert!(r.contains(needle), "missing `{needle}` in:\n{r}");
        }
    }

    #[test]
    fn assemble_body_orders_message_then_diagnostics() {
        let d = diag();
        let b = assemble_body(Some("run fails with code 7"), &d);
        let msg_at = b.find("run fails with code 7").unwrap();
        let diag_at = b.find("## Diagnostics").unwrap();
        assert!(msg_at < diag_at, "message should precede diagnostics");
        assert!(b.contains("\n---\n"));
        assert!(assemble_body(None, &d).starts_with("## Diagnostics"));
        assert!(assemble_body(Some("   "), &d).starts_with("## Diagnostics"));
    }

    #[test]
    fn prefilled_url_targets_repo_and_encodes() {
        let u = prefilled_url("o/n", "a b&c", "x\ny");
        assert!(u.starts_with("https://github.com/o/n/issues/new?title="));
        assert!(u.contains("a%20b%26c")); // space + ampersand escaped
        assert!(u.contains("x%0Ay")); // newline escaped
        assert!(!u.contains(' '));
    }

    #[test]
    fn resolve_repo_default_and_override() {
        assert_eq!(resolve_repo(None), crate::commands::upgrade::DEFAULT_REPO);
        assert_eq!(resolve_repo(Some("o/n")), "o/n");
    }

    #[test]
    fn issue_payload_shape() {
        let p = issue_payload("t", "b", &[]);
        assert_eq!(p["title"], "t");
        assert_eq!(p["body"], "b");
        assert!(p.get("labels").is_none());

        let p2 = issue_payload("t", "b", &["bug".to_string(), "vat".to_string()]);
        assert_eq!(p2["labels"], serde_json::json!(["bug", "vat"]));
    }
}
// HANDWRITE-END
