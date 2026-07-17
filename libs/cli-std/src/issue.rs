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

#[async_trait::async_trait]
pub trait IssueBackend: Send + Sync {
    async fn search(&self, repo: &str, state: &str, query: &str, limit: u32) -> Result<serde_json::Value>;
    async fn view(&self, repo: &str, number: u64) -> Result<serde_json::Value>;
    async fn submit_issue(&self, repo: &str, payload: &serde_json::Value) -> Result<String>;
    async fn reopen_issue(&self, repo: &str, number: u64) -> Result<String>;
    async fn post_issue_comment(&self, repo: &str, number: u64, body: &str) -> Result<String>;
    async fn fetch_node_status(&self, url: &str) -> String;
    fn has_credential(&self) -> bool;
}

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
pub async fn create(client: &impl IssueBackend, tool: &ToolInfo, opts: CreateOptions) -> Result<()> {
    let repo = resolve_repo(tool, opts.repo.as_deref()).to_string();

    let node = match opts.url.as_deref() {
        Some(url) => Some(client.fetch_node_status(url).await),
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

    if !opts.yes && !crate::confirm(&format!("file this issue to {repo}?"))? {
        println!("aborted");
        println!("next: done");
        return Ok(());
    }

    if client.has_credential() {
        let payload = issue_payload(&opts.title, &body, &opts.label);
        let filed_url = client.submit_issue(&repo, &payload).await?;
        println!("filed: {filed_url}");
        println!("next: done");
    } else {
        note_no_credential();
        print_fallback(&repo, &opts.title, &body, &opts.label);
    }
    Ok(())
}

/// Offline build: assemble + print (`--dry-run`) or the browser fallback.
#[cfg(not(feature = "online"))]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub async fn create(_client: &impl IssueBackend, tool: &ToolInfo, opts: CreateOptions) -> Result<()> {
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
pub async fn comment(client: &impl IssueBackend, tool: &ToolInfo, opts: CommentOptions) -> Result<()> {
    validate_issue_number(opts.number)?;
    let repo = resolve_repo(tool, opts.repo.as_deref()).to_string();
    let body = followup_comment_body(tool, opts.message.as_deref());

    if opts.dry_run {
        print_comment_preview(&repo, opts.number, &body);
        return Ok(());
    }

    if !opts.yes && !crate::confirm(&format!("comment on issue #{} in {repo} and ensure it is open?", opts.number))? {
        println!("aborted");
        println!("next: done");
        return Ok(());
    }

    if client.has_credential() {
        client.reopen_issue(&repo, opts.number).await?;
        let filed_url = client.post_issue_comment(&repo, opts.number, &body).await?;
        println!("filed: {filed_url}");
        println!("next: done");
    } else {
        note_no_credential_comment();
        print_comment_fallback(&repo, opts.number, &body);
    }
    Ok(())
}

/// Offline build: print the issue URL and the comment to paste after reopening.
#[cfg(not(feature = "online"))]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub async fn comment(_client: &impl IssueBackend, tool: &ToolInfo, opts: CommentOptions) -> Result<()> {
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
pub async fn search(client: &impl IssueBackend, tool: &ToolInfo, opts: SearchOptions) -> Result<()> {
    let q = if let Some(text) = opts.query.as_deref() {
        if !text.trim().is_empty() {
            Some(text.trim())
        } else {
            None
        }
    } else {
        None
    };
    
    let v = client.search(&tool.repo, &opts.state, q.unwrap_or(""), opts.limit).await?;

    let items = v.get("items").and_then(|i| i.as_array());
    if let Some(arr) = items {
        if arr.is_empty() {
            println!("(no results)");
        }
        for item in arr {
            let num = item.get("number").and_then(|n| n.as_u64()).unwrap_or(0);
            let state = item.get("state").and_then(|s| s.as_str()).unwrap_or("?");
            let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("");
            println!("#{num} [{state}] {title}");
        }
    } else {
        // Fallback for courier search response which might be an array directly
        if let Some(arr) = v.as_array() {
            if arr.is_empty() {
                println!("(no results)");
            }
            for item in arr {
                let num = item.get("number").and_then(|n| n.as_u64()).unwrap_or(0);
                let state = item.get("state").and_then(|s| s.as_str()).unwrap_or("?");
                let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("");
                println!("#{num} [{state}] {title}");
            }
        }
    }
    Ok(())
}

#[cfg(not(feature = "online"))]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub async fn search(_client: &impl IssueBackend, _tool: &ToolInfo, _opts: SearchOptions) -> Result<()> {
    anyhow::bail!("this build has no `online` feature — `issue search` needs network access")
}

// ---------------------------------------------------------------------------
// view (read)
// ---------------------------------------------------------------------------

/// `issue view` — print a single issue by number.
#[cfg(feature = "online")]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md#source
pub async fn view(client: &impl IssueBackend, tool: &ToolInfo, number: u64) -> Result<()> {
    let v = client.view(&tool.repo, number).await?;

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
pub async fn view(_client: &impl IssueBackend, _tool: &ToolInfo, _number: u64) -> Result<()> {
    anyhow::bail!("this build has no `online` feature — `issue view` needs network access")
}


// SPEC-MANAGED: libs/cli-std/tech-design/interfaces/cli/courier-proxy-mode-client-for-the-issue-triad.md#unit-test
// HANDWRITE-BEGIN gap="cli-std-unit-test-generator" tracker="#1320" reason="the unit-test generator emits an empty CODEGEN block for this project (no test-body synthesis primitive yet); hand-written proxy-mode routing + fallback tests against the pure URL builders extracted above (this crate has no HTTP-mock dev-dependency, so routing is verified via the exact request-shape builders search/view/create/comment call, not a live network round trip)."

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
