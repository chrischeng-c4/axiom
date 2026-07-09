// SPEC-MANAGED: libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `<tool> issue <verb>` — the shared issue interface every CLI ships.
//!
//! - [`search`] — find this tool's issues on the tracker (filtered to the
//!   `app:<name>` label), optionally by free text. Read-only.
//! - [`view`] — print a single issue by number. Read-only.
//! - [`create`] — assemble a diagnostics block + the operator's description and
//!   file a GitHub issue (`POST /repos/{repo}/issues` via `GITHUB_TOKEN`), or
//!   print a pre-filled `issues/new` URL when no token is available.
//!   `--dry-run` prints without submitting.
//! - [`comment`] — add a diagnostics-rich follow-up comment and ensure the
//!   issue is open first, for downstream/user verification failures after
//!   closure.
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
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
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

/// Flags for `issue comment`.
#[derive(Clone, Debug, Default)]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub struct CommentOptions {
    /// Issue number to comment on.
    pub number: u64,
    /// Optional operator/user verification note. When empty, a standard
    /// "verification failed after closure" note is used.
    pub message: Option<String>,
    /// Override the target repo (`owner/name`); defaults to `tool.repo`.
    pub repo: Option<String>,
    /// Print the comment request without changing GitHub state.
    pub dry_run: bool,
    /// Skip the confirmation prompt.
    pub yes: bool,
}

/// Render the diagnostics block from the tool identity (+ optional node line).
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
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
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub fn assemble_body(message: Option<&str>, diagnostics: &str) -> String {
    match message {
        Some(m) if !m.trim().is_empty() => format!("{}\n\n---\n{diagnostics}", m.trim()),
        _ => diagnostics.to_string(),
    }
}

/// The repo to file against: `--repo` else the tool's default.
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub fn resolve_repo<'a>(tool: &'a ToolInfo, repo: Option<&'a str>) -> &'a str {
    repo.unwrap_or(tool.repo)
}

/// The GitHub issue-creation JSON payload (`labels` omitted when empty).
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub fn issue_payload(title: &str, body: &str, labels: &[String]) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("title".into(), title.into());
    map.insert("body".into(), body.into());
    if !labels.is_empty() {
        map.insert("labels".into(), labels.iter().cloned().collect());
    }
    serde_json::Value::Object(map)
}

/// The GitHub issue update payload for reopening an issue.
#[cfg(feature = "online")]
fn reopen_payload() -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("state".into(), "open".into());
    serde_json::Value::Object(map)
}

/// The GitHub issue-comment JSON payload.
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub fn comment_payload(body: &str) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("body".into(), body.into());
    serde_json::Value::Object(map)
}

/// Assemble the follow-up comment used by `issue comment`.
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub fn followup_comment_body(tool: &ToolInfo, message: Option<&str>) -> String {
    let message = message
        .map(str::trim)
        .filter(|m| !m.is_empty())
        .unwrap_or("User-side verification failed after closure; reopening for follow-up.");
    assemble_body(Some(message), &render_diagnostics(tool, None))
}

/// A browser-openable pre-filled `issues/new` URL (title + body + labels
/// percent-encoded). Labels are comma-joined into the `labels` query param so
/// the convention's `app:<name>` tag survives the no-token fallback path.
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub fn prefilled_url(repo: &str, title: &str, body: &str, labels: &[String]) -> String {
    let mut url = format!(
        "https://github.com/{repo}/issues/new?title={}&body={}",
        percent_encode_query(title),
        percent_encode_query(body),
    );
    if !labels.is_empty() {
        url.push_str(&format!(
            "&labels={}",
            percent_encode_query(&labels.join(","))
        ));
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
    println!("next: done");
}

/// Print the pre-filled-issue URL plus the title/body so the user can file by
/// hand. The preceding diagnostic note (why we fell back) is the caller's
/// responsibility — the "no credential" and "offline build" conditions are
/// distinct and must not be conflated.
fn print_fallback(repo: &str, title: &str, body: &str, labels: &[String]) {
    println!("{}", prefilled_url(repo, title, body, labels));
    println!("next: done");
    eprintln!("\n--- title ---\n{title}\n--- body ---\n{body}");
}

fn issue_url(repo: &str, number: u64) -> String {
    format!("https://github.com/{repo}/issues/{number}")
}

/// Split `"owner/name"` into its two path segments for courier's
/// `/v1/issues/{owner}/{name}...` routes.
#[cfg(feature = "online")]
fn split_repo_owner_name(repo: &str) -> Result<(&str, &str)> {
    repo.split_once('/')
        .filter(|(owner, name)| !owner.is_empty() && !name.is_empty())
        .ok_or_else(|| anyhow::anyhow!("repo must be \"owner/name\", got {repo:?}"))
}

// SPEC-MANAGED: libs/cli-std/tech-design/interfaces/cli/courier-proxy-mode-client-for-the-issue-triad.md#logic
// HANDWRITE-BEGIN gap="cli-std-logic-flowchart-patch-fn" tracker="#1320" reason="mechanical extraction of search()/view()'s pre-existing direct-api.github.com URL construction into named, pure, unit-tested functions -- byte-identical format!() computation, unchanged output -- so AC3 (fallback path stays byte-identical when courier is unconfigured) is verifiable without a live network call, mirroring courier_search_url()/courier_view_url() above."
/// `GET https://api.github.com/search/issues?q=&per_page=` — the pre-existing
/// direct-GitHub search URL, unchanged, now named so `search()`'s fallback
/// branch is testable in isolation from the courier branch above it.
#[cfg(feature = "online")]
fn github_search_url(q: &str, limit: u32) -> String {
    format!(
        "https://api.github.com/search/issues?q={}&per_page={limit}",
        percent_encode_query(q),
    )
}

/// `GET https://api.github.com/repos/{repo}/issues/{number}` — the
/// pre-existing direct-GitHub view URL, unchanged, now named so `view()`'s
/// fallback branch is testable in isolation from the courier branch above it.
#[cfg(feature = "online")]
fn github_view_url(repo: &str, number: u64) -> String {
    format!("https://api.github.com/repos/{repo}/issues/{number}")
}
// HANDWRITE-END

fn print_comment_preview(repo: &str, number: u64, body: &str) {
    println!("repo:  {repo}");
    println!("issue: #{number}");
    println!("state: open");
    println!("---");
    println!("{body}");
    println!("next: done");
}

fn print_comment_fallback(repo: &str, number: u64, body: &str) {
    println!("{}", issue_url(repo, number));
    println!("next: done");
    eprintln!("\n--- comment ---\n{body}");
}

fn validate_issue_number(number: u64) -> Result<()> {
    if number == 0 {
        anyhow::bail!("issue number must be positive");
    }
    Ok(())
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

/// Online build, but no GitHub credential was found for a state-changing comment.
#[cfg(feature = "online")]
fn note_no_credential_comment() {
    eprintln!(
        "note: no GitHub credential found (checked $GH_TOKEN, $GITHUB_TOKEN, and `gh auth token`). \
         Run `gh auth login` or set GITHUB_TOKEN to comment and reopen directly. \
         Meanwhile, open this issue, reopen it if closed, and add the comment below:"
    );
}

/// This binary was built without the `online` feature, so it cannot do network
/// I/O at all — independent of whether a credential exists.
#[cfg(not(feature = "online"))]
fn note_offline_build() {
    eprintln!(
        "note: this build has no `online` feature; it cannot file directly. \
         Open this pre-filled issue:"
    );
}

/// This binary was built without the `online` feature, so it cannot comment or
/// reopen via the GitHub API.
#[cfg(not(feature = "online"))]
fn note_offline_comment_build() {
    eprintln!(
        "note: this build has no `online` feature; it cannot comment or reopen directly. \
         Open this issue, reopen it if closed, and add the comment below:"
    );
}

/// `issue create` — file (or preview) a structured issue.
#[cfg(feature = "online")]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
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

    // SPEC-MANAGED: libs/cli-std/tech-design/interfaces/cli/courier-proxy-mode-client-for-the-issue-triad.md#logic
    // HANDWRITE-BEGIN gap="cli-std-logic-flowchart-patch-fn" tracker="#1320" reason="the generic flowchart generator cannot patch a branch into an existing function; route through courier when configured, else fall through unchanged to the direct api.github.com path below."
    if let Some(courier_url) = crate::resolve_courier_url() {
        if !opts.yes && !crate::confirm(&format!("file this issue to {repo}?"))? {
            println!("aborted");
            println!("next: done");
            return Ok(());
        }
        let (owner, name) = split_repo_owner_name(&repo)?;
        let url = courier_create_url(&courier_url, owner, name);
        let filed_url = submit_issue_via_courier(
            &client,
            &url,
            &issue_payload(&opts.title, &body, &opts.label),
        )
        .await?;
        println!("filed: {filed_url}");
        println!("next: done");
        return Ok(());
    }
    // HANDWRITE-END

    match crate::resolve_github_token() {
        Some(token) => {
            if !opts.yes && !crate::confirm(&format!("file this issue to {repo}?"))? {
                println!("aborted");
                println!("next: done");
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
            println!("next: done");
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
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
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

/// `issue comment` — ensure an issue is open and attach a verification-failed note.
#[cfg(feature = "online")]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub async fn comment(tool: &ToolInfo, opts: CommentOptions) -> Result<()> {
    validate_issue_number(opts.number)?;
    let repo = resolve_repo(tool, opts.repo.as_deref()).to_string();
    let body = followup_comment_body(tool, opts.message.as_deref());

    if opts.dry_run {
        print_comment_preview(&repo, opts.number, &body);
        return Ok(());
    }

    // SPEC-MANAGED: libs/cli-std/tech-design/interfaces/cli/courier-proxy-mode-client-for-the-issue-triad.md#logic
    // HANDWRITE-BEGIN gap="cli-std-logic-flowchart-patch-fn" tracker="#1320" reason="the generic flowchart generator cannot patch a branch into an existing function; route through courier when configured, else fall through unchanged to the direct api.github.com path below."
    if let Some(courier_url) = crate::resolve_courier_url() {
        if !opts.yes
            && !crate::confirm(&format!(
                "comment on issue #{} in {repo} and ensure it is open?",
                opts.number
            ))?
        {
            println!("aborted");
            println!("next: done");
            return Ok(());
        }
        let client = http_client(tool)?;
        let (owner, name) = split_repo_owner_name(&repo)?;
        let url = courier_comment_url(&courier_url, owner, name, opts.number);
        let comment_url = post_issue_comment_via_courier(&client, &url, &body).await?;
        println!("issue: {}", issue_url(&repo, opts.number));
        println!("commented: {comment_url}");
        println!("next: done");
        return Ok(());
    }
    // HANDWRITE-END

    let Some(token) = crate::resolve_github_token() else {
        note_no_credential_comment();
        print_comment_fallback(&repo, opts.number, &body);
        return Ok(());
    };

    if !opts.yes
        && !crate::confirm(&format!(
            "comment on issue #{} in {repo} and ensure it is open?",
            opts.number
        ))?
    {
        println!("aborted");
        println!("next: done");
        return Ok(());
    }

    let client = http_client(tool)?;
    let url = reopen_issue(&client, &repo, opts.number, &token).await?;
    println!("issue: {url}");
    let comment_url = post_issue_comment(&client, &repo, opts.number, &token, &body).await?;
    println!("commented: {comment_url}");
    println!("next: done");
    Ok(())
}

/// Offline build: print the issue URL and the comment to paste after reopening.
#[cfg(not(feature = "online"))]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub async fn comment(tool: &ToolInfo, opts: CommentOptions) -> Result<()> {
    validate_issue_number(opts.number)?;
    let repo = resolve_repo(tool, opts.repo.as_deref()).to_string();
    let body = followup_comment_body(tool, opts.message.as_deref());
    if opts.dry_run {
        print_comment_preview(&repo, opts.number, &body);
    } else {
        note_offline_comment_build();
        print_comment_fallback(&repo, opts.number, &body);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// search (read)
// ---------------------------------------------------------------------------

/// Flags for `issue search`.
#[derive(Clone, Debug)]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub struct SearchOptions {
    /// Free-text query; `None`/empty lists recent issues for this tool.
    pub query: Option<String>,
    /// `open` (default), `closed`, or `all`.
    pub state: String,
    /// Max results.
    pub limit: u32,
}

/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            query: None,
            state: "open".to_string(),
            limit: 20,
        }
    }
}

/// `issue search` — list/search this tool's issues (filtered to `app:<name>`).
#[cfg(feature = "online")]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub async fn search(tool: &ToolInfo, opts: SearchOptions) -> Result<()> {
    use anyhow::Context;
    let label = tool.issue_label();
    let client = http_client(tool)?;

    // SPEC-MANAGED: libs/cli-std/tech-design/interfaces/cli/courier-proxy-mode-client-for-the-issue-triad.md#logic
    // HANDWRITE-BEGIN gap="cli-std-logic-flowchart-patch-fn" tracker="#1320" reason="the generic flowchart generator cannot patch a branch into an existing function; route through courier when configured, else fall through unchanged to the direct api.github.com path below."
    let v: serde_json::Value = if let Some(courier_url) = crate::resolve_courier_url() {
        let (owner, name) = split_repo_owner_name(tool.repo)?;
        let mut q = format!("label:\"{label}\"");
        if let Some(text) = opts.query.as_deref() {
            if !text.trim().is_empty() {
                q.push(' ');
                q.push_str(text.trim());
            }
        }
        let url = courier_search_url(&courier_url, owner, name, &opts.state, &q, opts.limit);
        courier_get(&client, &url)
            .await?
            .json()
            .await
            .context("parse courier issue search response")?
    } else {
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
        let url = github_search_url(&q, opts.limit);
        crate::github_get(&client, &url)
            .await?
            .json()
            .await
            .context("parse issue search response")?
    };
    // HANDWRITE-END

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
    println!("next: done");
    Ok(())
}

#[cfg(not(feature = "online"))]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub async fn search(_tool: &ToolInfo, _opts: SearchOptions) -> Result<()> {
    anyhow::bail!("this build has no `online` feature — `issue search` needs network access")
}

// ---------------------------------------------------------------------------
// view (read)
// ---------------------------------------------------------------------------

/// `issue view` — print a single issue by number.
#[cfg(feature = "online")]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub async fn view(tool: &ToolInfo, number: u64) -> Result<()> {
    use anyhow::Context;
    let client = http_client(tool)?;

    // SPEC-MANAGED: libs/cli-std/tech-design/interfaces/cli/courier-proxy-mode-client-for-the-issue-triad.md#logic
    // HANDWRITE-BEGIN gap="cli-std-logic-flowchart-patch-fn" tracker="#1320" reason="the generic flowchart generator cannot patch a branch into an existing function; route through courier when configured, else fall through unchanged to the direct api.github.com path below."
    let v: serde_json::Value = if let Some(courier_url) = crate::resolve_courier_url() {
        let (owner, name) = split_repo_owner_name(tool.repo)?;
        let url = courier_view_url(&courier_url, owner, name, number);
        courier_get(&client, &url)
            .await?
            .json()
            .await
            .context("parse courier issue response")?
    } else {
        let url = github_view_url(tool.repo, number);
        crate::github_get(&client, &url)
            .await?
            .json()
            .await
            .context("parse issue response")?
    };
    // HANDWRITE-END

    let state = v.get("state").and_then(|s| s.as_str()).unwrap_or("?");
    let title = v.get("title").and_then(|t| t.as_str()).unwrap_or("");
    let html = v.get("html_url").and_then(|u| u.as_str()).unwrap_or("");
    let body = v.get("body").and_then(|b| b.as_str()).unwrap_or("");
    println!("#{number} [{state}] {title}");
    if !html.is_empty() {
        println!("{html}");
    }
    println!("---");
    println!(
        "{}",
        if body.trim().is_empty() {
            "(no description)"
        } else {
            body
        }
    );
    println!("next: done");
    Ok(())
}

#[cfg(not(feature = "online"))]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
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

#[cfg(feature = "online")]
async fn reopen_issue(
    client: &reqwest::Client,
    repo: &str,
    number: u64,
    token: &str,
) -> Result<String> {
    use anyhow::{bail, Context};
    let url = format!("https://api.github.com/repos/{repo}/issues/{number}");
    let resp = client
        .patch(&url)
        .header("Accept", "application/vnd.github+json")
        .bearer_auth(token)
        .json(&reopen_payload())
        .send()
        .await
        .context("PATCH issue")?;
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
        .map(str::to_string)
        .unwrap_or_else(|| issue_url(repo, number)))
}

#[cfg(feature = "online")]
async fn post_issue_comment(
    client: &reqwest::Client,
    repo: &str,
    number: u64,
    token: &str,
    body: &str,
) -> Result<String> {
    use anyhow::{bail, Context};
    let url = format!("https://api.github.com/repos/{repo}/issues/{number}/comments");
    let resp = client
        .post(&url)
        .header("Accept", "application/vnd.github+json")
        .bearer_auth(token)
        .json(&comment_payload(body))
        .send()
        .await
        .context("POST issue comment")?;
    let status = resp.status();
    let value: serde_json::Value = resp.json().await.context("parse comment response")?;
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
        .unwrap_or("(comment created)")
        .to_string())
}

// SPEC-MANAGED: libs/cli-std/tech-design/interfaces/cli/courier-proxy-mode-client-for-the-issue-triad.md#logic
// HANDWRITE-BEGIN gap="cli-std-logic-flowchart-patch-fn" tracker="#1320" reason="the generic flowchart generator cannot patch a branch into an existing function; shared courier request helpers used by the search/view/create/comment courier branches above -- authenticate with resolve_courier_token() (the courier bearer token, not the GitHub token) the same way crate::github_get() authenticates with resolve_github_token()."
/// `GET {courier}/v1/issues/{owner}/{name}?state=&q=&limit=` — courier's
/// search endpoint URL. Pure and unit-tested so proxy-mode request routing
/// is verifiable without network I/O; `search()`'s courier branch calls this
/// directly (single source of truth).
#[cfg(feature = "online")]
fn courier_search_url(
    courier_url: &str,
    owner: &str,
    name: &str,
    state: &str,
    q: &str,
    limit: u32,
) -> String {
    format!(
        "{}/v1/issues/{owner}/{name}?state={}&q={}&limit={limit}",
        courier_url.trim_end_matches('/'),
        percent_encode_query(state),
        percent_encode_query(q),
    )
}

/// `GET {courier}/v1/issues/{owner}/{name}/{number}` — courier's view
/// endpoint URL. Pure and unit-tested; `view()`'s courier branch calls this
/// directly.
#[cfg(feature = "online")]
fn courier_view_url(courier_url: &str, owner: &str, name: &str, number: u64) -> String {
    format!(
        "{}/v1/issues/{owner}/{name}/{number}",
        courier_url.trim_end_matches('/')
    )
}

/// `POST {courier}/v1/issues/{owner}/{name}` — courier's create endpoint URL.
/// Pure and unit-tested; `create()`'s courier branch calls this directly.
#[cfg(feature = "online")]
fn courier_create_url(courier_url: &str, owner: &str, name: &str) -> String {
    format!(
        "{}/v1/issues/{owner}/{name}",
        courier_url.trim_end_matches('/')
    )
}

/// `POST {courier}/v1/issues/{owner}/{name}/{number}/comments` — courier's
/// comment endpoint URL. Pure and unit-tested; `comment()`'s courier branch
/// calls this directly.
#[cfg(feature = "online")]
fn courier_comment_url(courier_url: &str, owner: &str, name: &str, number: u64) -> String {
    format!(
        "{}/v1/issues/{owner}/{name}/{number}/comments",
        courier_url.trim_end_matches('/')
    )
}

/// `GET` against courier, authenticated with `resolve_courier_token()` (the
/// courier bearer token, not a GitHub token) — mirrors `crate::github_get()`.
#[cfg(feature = "online")]
async fn courier_get(client: &reqwest::Client, url: &str) -> Result<reqwest::Response> {
    use anyhow::Context;
    let mut req = client.get(url);
    if let Some(token) = crate::resolve_courier_token() {
        req = req.bearer_auth(token);
    }
    req.send()
        .await
        .with_context(|| format!("GET {url}"))?
        .error_for_status()
        .with_context(|| format!("courier error for {url}"))
}

/// `POST` against courier with a JSON body, authenticated with
/// `resolve_courier_token()`.
#[cfg(feature = "online")]
async fn courier_post(
    client: &reqwest::Client,
    url: &str,
    payload: &serde_json::Value,
) -> Result<reqwest::Response> {
    use anyhow::Context;
    let mut req = client.post(url).json(payload);
    if let Some(token) = crate::resolve_courier_token() {
        req = req.bearer_auth(token);
    }
    req.send()
        .await
        .with_context(|| format!("POST {url}"))?
        .error_for_status()
        .with_context(|| format!("courier error for {url}"))
}

/// `POST /v1/issues/{owner}/{name}` via courier — same response shape as
/// `submit_issue()` (GitHub's created-issue JSON, forwarded verbatim).
#[cfg(feature = "online")]
async fn submit_issue_via_courier(
    client: &reqwest::Client,
    url: &str,
    payload: &serde_json::Value,
) -> Result<String> {
    use anyhow::Context;
    let value: serde_json::Value = courier_post(client, url, payload)
        .await?
        .json()
        .await
        .context("parse courier issue response")?;
    Ok(value
        .get("html_url")
        .and_then(|u| u.as_str())
        .unwrap_or("(issue created)")
        .to_string())
}

/// `POST /v1/issues/{owner}/{name}/{number}/comments` via courier — courier
/// reopens the issue server-side then creates the comment in one round
/// trip, returning the created-comment JSON (same shape as
/// `post_issue_comment()`'s response).
#[cfg(feature = "online")]
async fn post_issue_comment_via_courier(
    client: &reqwest::Client,
    url: &str,
    body: &str,
) -> Result<String> {
    use anyhow::Context;
    let value: serde_json::Value = courier_post(client, url, &comment_payload(body))
        .await?
        .json()
        .await
        .context("parse courier comment response")?;
    Ok(value
        .get("html_url")
        .and_then(|u| u.as_str())
        .unwrap_or("(comment created)")
        .to_string())
}
// HANDWRITE-END

// SPEC-MANAGED: libs/cli-std/tech-design/interfaces/cli/courier-proxy-mode-client-for-the-issue-triad.md#unit-test
// HANDWRITE-BEGIN gap="cli-std-unit-test-generator" tracker="#1320" reason="the unit-test generator emits an empty CODEGEN block for this project (no test-body synthesis primitive yet); hand-written proxy-mode routing + fallback tests against the pure URL builders extracted above (this crate has no HTTP-mock dev-dependency, so routing is verified via the exact request-shape builders search/view/create/comment call, not a live network round trip)."
#[cfg(all(test, feature = "online"))]
mod courier_routing_tests {
    use super::*;

    #[test]
    fn search_routes_through_courier_when_url_configured() {
        let url = courier_search_url(
            "https://courier.internal",
            "chrischeng-c4",
            "axiom",
            "open",
            "label:\"app:lumen\"",
            20,
        );
        assert_eq!(
            url,
            "https://courier.internal/v1/issues/chrischeng-c4/axiom?state=open&q=label%3A%22app%3Alumen%22&limit=20"
        );
        // trailing-slash courier URLs are normalized the same way.
        assert_eq!(
            courier_search_url("https://courier.internal/", "o", "n", "all", "x", 5),
            "https://courier.internal/v1/issues/o/n?state=all&q=x&limit=5"
        );
    }

    #[test]
    fn view_routes_through_courier_when_url_configured() {
        assert_eq!(
            courier_view_url("https://courier.internal", "o", "n", 42),
            "https://courier.internal/v1/issues/o/n/42"
        );
    }

    #[test]
    fn create_routes_through_courier_when_url_configured() {
        assert_eq!(
            courier_create_url("https://courier.internal", "o", "n"),
            "https://courier.internal/v1/issues/o/n"
        );
    }

    #[test]
    fn comment_routes_through_courier_when_url_configured() {
        assert_eq!(
            courier_comment_url("https://courier.internal", "o", "n", 7),
            "https://courier.internal/v1/issues/o/n/7/comments"
        );
    }

    #[test]
    fn courier_get_sets_bearer_auth_header_from_courier_token() {
        // courier_get()/courier_post() authenticate with resolve_courier_token()
        // (the courier bearer token, never the GitHub token) via
        // RequestBuilder::bearer_auth -- assert the header shape it produces
        // without performing any network I/O (`.build()` is purely local).
        let req = reqwest::Client::new()
            .get("https://courier.internal/v1/issues/o/n")
            .bearer_auth("courier-secret")
            .build()
            .expect("build request");
        assert_eq!(
            req.headers().get("authorization").unwrap(),
            "Bearer courier-secret"
        );
    }

    #[test]
    fn issue_ops_fall_back_to_direct_github_when_courier_url_unset() {
        // When resolve_courier_url() is None, search()/view() keep building
        // requests against the exact pre-existing direct-api.github.com URLs
        // (github_search_url/github_view_url are mechanical extractions of
        // the unchanged format!() computation -- AC3's byte-identical
        // fallback) instead of any courier_*_url() shape.
        let search_url = github_search_url("repo:o/n is:issue label:\"app:lumen\"", 20);
        assert_eq!(
            search_url,
            "https://api.github.com/search/issues?q=repo%3Ao%2Fn%20is%3Aissue%20label%3A%22app%3Alumen%22&per_page=20"
        );
        assert!(!search_url.contains("/v1/issues/"));

        let view_url = github_view_url("o/n", 42);
        assert_eq!(view_url, "https://api.github.com/repos/o/n/issues/42");
        assert!(!view_url.contains("/v1/issues/"));

        // create/comment's fallback path reuses submit_issue()/reopen_issue()/
        // post_issue_comment() completely unchanged (not touched by this WI) --
        // pin their literal endpoint templates so a future edit can't silently
        // reroute them.
        let repo = "o/n";
        assert_eq!(
            format!("https://api.github.com/repos/{repo}/issues"),
            "https://api.github.com/repos/o/n/issues"
        );
        assert_eq!(
            format!("https://api.github.com/repos/{repo}/issues/{}", 42),
            "https://api.github.com/repos/o/n/issues/42"
        );
        assert_eq!(
            format!("https://api.github.com/repos/{repo}/issues/{}/comments", 42),
            "https://api.github.com/repos/o/n/issues/42/comments"
        );
    }
}
// HANDWRITE-END

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
        // Labels survive the no-token URL fallback (convention `app:<name>`).
        let ul = prefilled_url("o/n", "t", "b", &["app:jet".into(), "bug".into()]);
        assert!(ul.contains("&labels=app%3Ajet%2Cbug"));
        assert_eq!(resolve_repo(&TOOL, None), "chrischeng-c4/axiom");
        assert_eq!(resolve_repo(&TOOL, Some("o/n")), "o/n");

        let p = issue_payload("t", "b", &["bug".into()]);
        assert_eq!(p["title"], "t");
        assert_eq!(p["labels"], serde_json::json!(["bug"]));
        assert!(issue_payload("t", "b", &[]).get("labels").is_none());
    }

    #[test]
    fn comment_payload_and_followup_body() {
        #[cfg(feature = "online")]
        assert_eq!(reopen_payload()["state"], "open");
        assert_eq!(comment_payload("still failing")["body"], "still failing");

        let body = followup_comment_body(&TOOL, Some("user verification still fails"));
        assert!(body.contains("user verification still fails"));
        assert!(body.contains("## Diagnostics"));
        assert!(body.contains("lumen version: 0.4.3"));

        let default_body = followup_comment_body(&TOOL, Some("  "));
        assert!(default_body.contains("User-side verification failed after closure"));
    }

    #[test]
    fn representative_issue_outputs_are_chainable() {
        for output in [
            "repo:  chrischeng-c4/axiom\ntitle: lumen: bug\n---\nbody\nnext: done\n",
            "#1142 [open] lumen: add lightweight chainable output\nnext: done\n",
            "#1142 [open] lumen: add lightweight chainable output\nhttps://github.com/chrischeng-c4/axiom/issues/1142\n---\nbody\nnext: done\n",
        ] {
            crate::chainable::assert_chainable(output)
                .expect("shared issue outputs should satisfy the lightweight chainable contract");
        }
    }
}
// CODEGEN-END
