// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/backends/github_preamble_source.md#source
// CODEGEN-BEGIN
//! GitHub backend — shells out to the `gh` CLI.
//!
//! The CRRR write contract round-trips through GitHub's native attributes
//! (title, state, body, labels) — see `crate::issues::labels` for the
//! label-prefix scheme that encodes `phase`, `review_count`, `flagged_sections`,
//! `fill_retry_count`, `ship_status`, and `ship_commit` as labels on the
//! GitHub issue. `slug:*` labels are treated as legacy aliases; the GitHub
//! issue number is the canonical identity.
//!
//! Authentication is delegated to the `gh` CLI (user must have run
//! `gh auth login` beforehand).

use crate::issues::backend::IssueBackend;
use crate::issues::labels;
use crate::issues::types::{Issue, IssueFilter, IssuePatch, IssueState, IssueType};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::process::Command;
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/backends/github.md#schema
// CODEGEN-BEGIN
/// Issue backend that calls the `gh` CLI.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/backends/github.md#schema
pub struct GitHubBackend {
    /// Optional `owner/repo` slug. None = use CWD-detected repo.
    repo: Option<String>,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/backends/github_runtime_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/backends/github_runtime_source.md#source
impl GitHubBackend {
    pub fn new(repo: Option<String>) -> Self {
        Self { repo }
    }

    /// Construct with optional self-hosted host (GitHub Enterprise).
    /// Currently `host` is accepted but not yet threaded into the `gh` CLI
    /// invocation — `gh` already honours `GH_HOST`. Stored for future REST use.
    pub fn with_host(repo: Option<String>, host: Option<String>) -> Self {
        if let Some(h) = host.as_deref() {
            // Setting GH_HOST scopes subsequent gh CLI calls to the enterprise instance.
            std::env::set_var("GH_HOST", h);
        }
        Self { repo }
    }

    fn gh_args_prefix(&self) -> Vec<String> {
        let mut args = vec!["issue".to_string()];
        if let Some(r) = &self.repo {
            args.push("--repo".into());
            args.push(r.clone());
        }
        args
    }

    fn rest_repo_endpoint(&self, suffix: &str) -> String {
        let repo = self.repo.as_deref().unwrap_or("{owner}/{repo}");
        format!("repos/{repo}/issues{suffix}")
    }

    fn rest_create_args(&self, title: &str, body: &str, label_names: &[String]) -> Vec<String> {
        let mut args = vec![
            "api".to_string(),
            "-X".to_string(),
            "POST".to_string(),
            self.rest_repo_endpoint(""),
            "-f".to_string(),
            format!("title={title}"),
            "-f".to_string(),
            format!("body={body}"),
        ];
        for label in label_names {
            args.push("-f".to_string());
            args.push(format!("labels[]={label}"));
        }
        args
    }

    fn rest_patch_args(
        &self,
        number: u64,
        title: Option<&str>,
        body: Option<&str>,
        labels: Option<&[String]>,
        state: Option<&str>,
    ) -> Vec<String> {
        let mut args = vec![
            "api".to_string(),
            "-X".to_string(),
            "PATCH".to_string(),
            self.rest_repo_endpoint(&format!("/{number}")),
        ];
        if let Some(title) = title {
            args.push("-f".to_string());
            args.push(format!("title={title}"));
        }
        if let Some(body) = body {
            args.push("-f".to_string());
            args.push(format!("body={body}"));
        }
        if let Some(labels) = labels {
            for label in labels {
                args.push("-f".to_string());
                args.push(format!("labels[]={label}"));
            }
        }
        if let Some(state) = state {
            args.push("-f".to_string());
            args.push(format!("state={state}"));
        }
        args
    }

    /// Resolve a legacy slug alias to a GitHub issue number by querying the
    /// `slug:<slug>` label. Returns `None` if no issue carries that label.
    fn resolve_slug(&self, slug: &str) -> Result<Option<u64>> {
        let mut args = self.gh_args_prefix();
        args.push("list".into());
        args.push("--label".into());
        args.push(format!("{}{}", labels::SLUG_PREFIX, slug));
        args.push("--state".into());
        args.push("all".into());
        args.push("--limit".into());
        args.push("2".into());
        args.push("--json".into());
        args.push("number".into());

        let output = run_gh(&args)?;
        let json: Value = serde_json::from_str(&output)
            .with_context(|| format!("failed to parse gh slug-resolve JSON: {}", output))?;
        let arr = json
            .as_array()
            .ok_or_else(|| anyhow!("gh returned non-array on slug-resolve: {}", output))?;
        if arr.is_empty() {
            return Ok(None);
        }
        if arr.len() > 1 {
            // More than one issue carrying the slug — caller should treat as
            // ambiguous. For now we return the lowest number deterministically.
        }
        let n = arr
            .iter()
            .filter_map(|v| v["number"].as_u64())
            .min()
            .ok_or_else(|| anyhow!("slug:{slug} matched but no number field"))?;
        Ok(Some(n))
    }

    /// Ensure all `labels` exist on the repo, creating any that don't via
    /// `gh label create --force`. Required because `gh issue edit --add-label`
    /// fails if the label is unknown.
    fn ensure_labels_exist(&self, label_names: &[String]) -> Result<()> {
        for name in label_names {
            // `--force` makes the call idempotent (creates if missing,
            // updates color/desc if exists).
            let mut args: Vec<String> = vec!["label".into(), "create".into(), name.clone()];
            if let Some(r) = &self.repo {
                args.push("--repo".into());
                args.push(r.clone());
            }
            args.push("--force".into());
            // Silent on failure — the issue-edit call below will surface a
            // clearer error if the label still cannot be applied.
            let _ = gh_command().args(&args).output();
        }
        Ok(())
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/backends/github_runtime_source.md#source
impl IssueBackend for GitHubBackend {
    fn name(&self) -> &'static str {
        "github"
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R8 — writable now that create/update/close are implemented
    fn is_writable(&self) -> bool {
        true
    }

    async fn list(&self, filter: &IssueFilter) -> Result<Vec<Issue>> {
        let mut args = self.gh_args_prefix();
        args.push("list".into());
        args.push("--limit".into());
        args.push("500".into());
        args.push("--json".into());
        args.push("number,title,state,labels,author,createdAt,updatedAt,url,body".into());

        // Translate filter to gh flags where possible (server-side).
        match filter.state {
            Some(IssueState::Open) => {
                args.push("--state".into());
                args.push("open".into());
            }
            Some(IssueState::Closed) => {
                args.push("--state".into());
                args.push("closed".into());
            }
            Some(IssueState::Draft) | None => {
                args.push("--state".into());
                args.push("all".into());
            }
        }
        if let Some(label) = &filter.label {
            args.push("--label".into());
            args.push(label.clone());
        }
        if let Some(author) = &filter.author {
            args.push("--author".into());
            args.push(author.clone());
        }

        let output = match run_gh(&args) {
            Ok(output) => output,
            Err(err) => {
                let msg = err.to_string();
                if !is_graphql_or_auth_error(&msg) {
                    return Err(err);
                }
                self.list_via_rest(filter).with_context(|| {
                    format!("gh issue list failed, REST fallback also failed: {msg}")
                })?
            }
        };
        let json: Value =
            serde_json::from_str(&output).context("Failed to parse gh issue list JSON")?;
        let items = json
            .as_array()
            .ok_or_else(|| anyhow!("gh returned non-array: {}", output))?;

        let mut issues: Vec<Issue> = items
            .iter()
            .filter(|v| v.get("pull_request").is_none())
            .map(|v| parse_gh_issue(v))
            .collect::<Result<_>>()?;

        // Apply client-side filters that gh doesn't natively support
        // (issue_type filter — gh has no type flag, we filter by label).
        if filter.issue_type.is_some() {
            issues.retain(|i| filter.matches(i));
        }

        // Deterministic order: by GitHub issue number.
        issues.sort_by_key(|i| i.github_id.unwrap_or(0));
        Ok(issues)
    }

    /// Look up an issue by numeric GitHub issue number OR legacy slug alias.
    async fn get(&self, id: &str) -> Result<Option<Issue>> {
        let number: u64 = match id.parse() {
            Ok(n) => n,
            Err(_) => match self.resolve_slug(id)? {
                Some(n) => n,
                None => return Ok(None),
            },
        };

        let mut args = self.gh_args_prefix();
        args.push("view".into());
        args.push(number.to_string());
        args.push("--json".into());
        args.push("number,title,state,labels,author,createdAt,updatedAt,url,body".into());

        let output = match run_gh(&args) {
            Ok(s) => s,
            Err(e) => {
                // Treat "not found" as None rather than an error.
                let msg = e.to_string();
                if msg.contains("no issues") || msg.contains("404") {
                    return Ok(None);
                }
                if is_graphql_or_auth_error(&msg) {
                    return self.get_via_rest(number).map(Some).or_else(|fallback| {
                        if fallback.to_string().contains("404") {
                            Ok(None)
                        } else {
                            Err(fallback.context(format!(
                                "gh issue view failed, REST fallback also failed: {msg}"
                            )))
                        }
                    });
                }
                return Err(e);
            }
        };

        let json: Value =
            serde_json::from_str(&output).context("Failed to parse gh issue view JSON")?;
        Ok(Some(parse_gh_issue(&json)?))
    }

    /// Persist the full issue: round-trip title, state, body, and CRRR-state
    /// labels. Labels not in the score-managed set are preserved.
    async fn write(&self, issue: &Issue) -> Result<()> {
        // Resolve target issue number — prefer github_id, then numeric slug,
        // then legacy slug label.
        let number: u64 = match issue.github_id {
            Some(n) => n,
            None => match issue.slug.parse::<u64>() {
                Ok(n) => n,
                Err(_) => self.resolve_slug(&issue.slug)?.ok_or_else(|| {
                    anyhow!(
                        "GitHubBackend::write: cannot locate issue (slug='{}', no github_id)",
                        issue.slug
                    )
                })?,
            },
        };

        // Fetch current state for diffing. Prefer the legacy gh issue path,
        // but fall back to REST when gh issue uses a GraphQL path that the
        // current token cannot access.
        let mut view_args = self.gh_args_prefix();
        view_args.push("view".into());
        view_args.push(number.to_string());
        view_args.push("--json".into());
        view_args.push("number,title,state,labels,author,createdAt,updatedAt,url,body".into());
        let current = match run_gh(&view_args) {
            Ok(view_out) => {
                let view: Value = serde_json::from_str(&view_out)
                    .with_context(|| format!("failed to parse gh view JSON: {}", view_out))?;
                parse_gh_issue(&view)?
            }
            Err(err) => {
                let msg = err.to_string();
                if !is_graphql_or_auth_error(&msg) {
                    return Err(err);
                }
                self.get_via_rest(number).with_context(|| {
                    format!("gh issue view failed, REST fallback also failed: {msg}")
                })?
            }
        };

        let current_title = current.title.clone();
        let current_body = current.body.clone();
        let current_labels = current.labels.clone();

        // Compute desired label set + diff.
        let desired_labels = labels::encode_labels(issue);
        let (to_add, to_remove) = labels::diff_labels(&current_labels, &desired_labels);

        // Pre-create any unknown labels so --add-label doesn't fail.
        if !to_add.is_empty() {
            self.ensure_labels_exist(&to_add)?;
        }

        // Edit title / body / labels in one REST PATCH when anything changed.
        // PATCH receives the full next label set so user labels are preserved
        // while stale AW-managed labels are removed.
        let title_changed = issue.title != current_title;
        let body_changed = issue.body != current_body;
        let labels_changed = !to_add.is_empty() || !to_remove.is_empty();

        // State transitions are handled by REST too, avoiding gh issue's
        // GraphQL path for reopen/close.
        let desired_open = matches!(issue.state, IssueState::Open | IssueState::Draft);
        let currently_open = matches!(current.state, IssueState::Open | IssueState::Draft);
        let state_changed = desired_open != currently_open;

        if title_changed || body_changed || labels_changed || state_changed {
            let mut next_labels = current_labels.clone();
            next_labels.retain(|label| !to_remove.contains(label));
            for label in to_add {
                if !next_labels.contains(&label) {
                    next_labels.push(label);
                }
            }
            let title = title_changed.then_some(issue.title.as_str());
            let body = body_changed.then_some(issue.body.as_str());
            let labels = labels_changed.then_some(next_labels.as_slice());
            let state = state_changed.then_some(if desired_open { "open" } else { "closed" });
            run_gh_api(&self.rest_patch_args(number, title, body, labels, state))?;
        }

        Ok(())
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R1
    async fn create(&self, issue: &Issue) -> Result<Issue> {
        // Create the issue before assigning the canonical slug. GitHub's
        // issue number is the canonical identity, so a title-derived slug
        // label must not be part of the create preconditions.
        let mut staged = issue.clone();
        staged.slug.clear();

        // Combine user labels with managed CRRR labels (phase/review/etc.).
        let desired = labels::encode_labels(&staged);
        if !desired.is_empty() {
            self.ensure_labels_exist(&desired)?;
        }

        let output = run_gh_api(&self.rest_create_args(&staged.title, &staged.body, &desired))?;
        let json: Value =
            serde_json::from_str(&output).context("Failed to parse gh api create issue JSON")?;
        let created = parse_gh_issue(&json)?;

        staged.github_id = created.github_id;
        staged.url = created.url;
        staged.author = created.author;
        staged.labels = created.labels;
        staged.created_at = created.created_at;
        staged.updated_at = created.updated_at;
        staged.review_count = created.review_count;
        staged.flagged_sections = created.flagged_sections;
        staged.fill_retry_count = created.fill_retry_count;
        staged.ship_status = created.ship_status;
        staged.ship_commit = created.ship_commit;
        staged.state = IssueState::Open;
        if let Some(n) = staged.github_id {
            staged.slug = n.to_string();
        }
        Ok(staged)
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R2
    async fn update(&self, id: &str, patch: &IssuePatch) -> Result<Issue> {
        // Resolve to numeric id first.
        let number: u64 = match id.parse::<u64>() {
            Ok(n) => n,
            Err(_) => self
                .resolve_slug(id)?
                .ok_or_else(|| anyhow!("issue '{}' not found on GitHub", id))?,
        };

        // Strategy: load current, apply patch in-memory, write() back. Keeps
        // managed-label semantics consistent with the full write contract.
        let mut current = self
            .get(&number.to_string())
            .await?
            .ok_or_else(|| anyhow!("issue #{} not found", number))?;
        patch.apply(&mut current);
        self.write(&current).await?;

        self.get(&number.to_string())
            .await?
            .ok_or_else(|| anyhow!("issue #{} not found after update", number))
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R3
    async fn close(&self, id: &str, reason: Option<&str>) -> Result<()> {
        let number: u64 = match id.parse::<u64>() {
            Ok(n) => n,
            Err(_) => self
                .resolve_slug(id)?
                .ok_or_else(|| anyhow!("issue '{}' not found on GitHub", id))?,
        };

        run_gh_api(&self.rest_patch_args(number, None, None, None, Some("closed")))?;
        if let Some(comment) = reason {
            let args = vec![
                "api".to_string(),
                "-X".to_string(),
                "POST".to_string(),
                self.rest_repo_endpoint(&format!("/{number}/comments")),
                "-f".to_string(),
                format!("body={comment}"),
            ];
            run_gh_api(&args)?;
        }
        Ok(())
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R4
    async fn search(&self, query: &str) -> Result<Vec<Issue>> {
        let mut args = self.gh_args_prefix();
        args.push("list".into());
        args.push("--search".into());
        args.push(query.to_string());
        args.push("--limit".into());
        args.push("100".into());
        args.push("--json".into());
        args.push("number,title,state,labels,author,createdAt,updatedAt,url,body".into());
        args.push("--state".into());
        args.push("all".into());

        let output = run_gh(&args)?;
        let json: serde_json::Value =
            serde_json::from_str(&output).context("Failed to parse gh issue list --search JSON")?;
        let items = json
            .as_array()
            .ok_or_else(|| anyhow!("gh returned non-array: {}", output))?;

        items.iter().map(|v| parse_gh_issue(v)).collect()
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/backends/github_runtime_source.md#source
impl GitHubBackend {
    fn list_via_rest(&self, filter: &IssueFilter) -> Result<String> {
        let mut args = vec![
            "api".to_string(),
            "-X".to_string(),
            "GET".to_string(),
            self.rest_repo_endpoint("").to_string(),
            "-f".to_string(),
            "per_page=100".to_string(),
            "-f".to_string(),
            format!(
                "state={}",
                match filter.state {
                    Some(IssueState::Open) => "open",
                    Some(IssueState::Closed) => "closed",
                    Some(IssueState::Draft) | None => "all",
                }
            ),
        ];
        if let Some(label) = &filter.label {
            args.push("-f".to_string());
            args.push(format!("labels={label}"));
        }
        run_gh_api(&args)
    }

    fn get_via_rest(&self, number: u64) -> Result<Issue> {
        let args = vec![
            "api".to_string(),
            "-X".to_string(),
            "GET".to_string(),
            self.rest_repo_endpoint(&format!("/{number}")),
        ];
        let output = run_gh_api(&args)?;
        let json: Value =
            serde_json::from_str(&output).context("Failed to parse gh api issue JSON")?;
        parse_gh_issue(&json)
    }
}

/// Parse a single `gh` JSON issue object into our `Issue` type. The GitHub
/// issue number is the canonical local slug; any `slug:*` label is decoded
/// only as a legacy lookup alias.
fn parse_gh_issue(v: &Value) -> Result<Issue> {
    let number = v["number"]
        .as_u64()
        .ok_or_else(|| anyhow!("gh issue missing number: {}", v))?;
    let title = v["title"].as_str().unwrap_or("(untitled)").to_string();
    let body = v["body"].as_str().unwrap_or("").replace("\\`", "`");

    let state_raw = v["state"].as_str().unwrap_or("OPEN");
    let state = IssueState::parse_loose(state_raw).unwrap_or(IssueState::Open);

    let raw_labels: Vec<String> = v["labels"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|l| l["name"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let issue_type = IssueType::from_labels(&raw_labels);
    let decoded = labels::decode_labels(&raw_labels);
    let canonical_slug = number.to_string();

    let author = v["author"]["login"]
        .as_str()
        .or_else(|| v["user"]["login"].as_str())
        .map(String::from);
    let created_at = v["createdAt"]
        .as_str()
        .or_else(|| v["created_at"].as_str())
        .map(String::from);
    let updated_at = v["updatedAt"]
        .as_str()
        .or_else(|| v["updated_at"].as_str())
        .map(String::from);
    let url = v["url"]
        .as_str()
        .or_else(|| v["html_url"].as_str())
        .map(String::from);

    Ok(Issue {
        issue_type,
        title,
        state,
        id: None,
        github_id: Some(number),
        gitlab_id: None,
        url,
        author,
        labels: raw_labels,
        created_at,
        updated_at,
        slug: canonical_slug,
        body,
        related: vec![],
        implements: vec![],
        phase: decoded.phase.clone(),
        branch: None,
        target_branch: None,
        git_workflow: None,
        change_id: None,
        iteration: None,
        current_task_id: None,
        impl_spec_phase: None,
        task_revisions: None,
        revision_counts: None,
        last_action: None,
        session_id: None,
        validation_errors: vec![],
        review_count: decoded.review_count,
        flagged_sections: decoded.flagged_sections.clone(),
        fill_retry_count: decoded.fill_retry_count,
        ship_status: decoded.ship_status,
        ship_commit: decoded.ship_commit.clone(),
        regen_verified_at: None,
    })
}

fn run_gh(args: &[String]) -> Result<String> {
    let output = gh_command()
        .args(args)
        .output()
        .context("Failed to execute gh — is the GitHub CLI installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh {:?} failed: {}", args, stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run_gh_api(args: &[String]) -> Result<String> {
    match run_gh_api_direct(args) {
        Ok(output) => Ok(output),
        Err(err) => {
            let msg = err.to_string();
            if !(is_graphql_or_auth_error(&msg) || msg.contains("HTTP 404")) {
                return Err(err);
            }
            run_gh_api_via_shell(args)
                .with_context(|| format!("direct gh api failed, shell retry failed: {msg}"))
        }
    }
}

fn run_gh_api_direct(args: &[String]) -> Result<String> {
    let mut command = gh_command();
    command.args(args);
    if std::env::var_os("GH_TOKEN").is_none() {
        if let Some(token) = gh_auth_token() {
            command.env("GH_TOKEN", token);
        }
    }
    let output = command
        .output()
        .context("Failed to execute gh — is the GitHub CLI installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh {:?} failed: {}", args, stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run_gh_api_via_shell(args: &[String]) -> Result<String> {
    let shell = if std::path::Path::new("/bin/zsh").exists() {
        "/bin/zsh".to_string()
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    };
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(shell_quote("gh"));
    parts.extend(args.iter().map(|arg| shell_quote(arg)));
    let command_line = parts.join(" ");
    let mut command = Command::new(shell);
    crate::cli::shell_env::apply_default_shell_env(&mut command);
    let output = command
        .arg("-lc")
        .arg(command_line)
        .output()
        .context("Failed to execute shell gh api retry")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh {:?} failed via shell: {}", args, stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn gh_auth_token() -> Option<String> {
    let output = gh_command().args(["auth", "token"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!token.is_empty()).then_some(token)
}

fn gh_command() -> Command {
    let mut command = Command::new("gh");
    crate::cli::shell_env::apply_default_shell_env(&mut command);
    command
}

fn is_graphql_or_auth_error(message: &str) -> bool {
    message.contains("GraphQL")
        || message.contains("HTTP 401")
        || message.contains("Requires authentication")
        || message.contains("could not resolve to a Repository")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_gh_issue_uses_number_as_canonical_slug() {
        let value = json!({
            "number": 1887,
            "title": "bug(score): slug round trip",
            "state": "OPEN",
            "labels": [
                {"name": "type:bug"},
                {"name": "phase:created"},
                {"name": "slug:bug-slug-round-trip-broken-local-cache-slug-d"}
            ],
            "author": {"login": "tester"},
            "createdAt": "2026-05-11T00:00:00Z",
            "updatedAt": "2026-05-11T00:00:00Z",
            "url": "https://github.com/owner/repo/issues/1887",
            "body": "body"
        });

        let issue = parse_gh_issue(&value).unwrap();

        assert_eq!(issue.github_id, Some(1887));
        assert_eq!(issue.slug, "1887");
        assert_eq!(issue.phase.as_deref(), Some("created"));
    }

    #[test]
    fn parse_rest_issue_shape_uses_user_and_snake_case_timestamps() {
        let value = json!({
            "number": 1888,
            "title": "bug(score): rest fallback",
            "state": "open",
            "labels": [{"name": "type:bug"}],
            "user": {"login": "rest-user"},
            "created_at": "2026-05-11T00:00:00Z",
            "updated_at": "2026-05-11T01:00:00Z",
            "html_url": "https://github.com/owner/repo/issues/1888",
            "body": "rest body"
        });

        let issue = parse_gh_issue(&value).unwrap();

        assert_eq!(issue.github_id, Some(1888));
        assert_eq!(issue.author.as_deref(), Some("rest-user"));
        assert_eq!(
            issue.url.as_deref(),
            Some("https://github.com/owner/repo/issues/1888")
        );
        assert_eq!(issue.created_at.as_deref(), Some("2026-05-11T00:00:00Z"));
        assert_eq!(issue.updated_at.as_deref(), Some("2026-05-11T01:00:00Z"));
    }

    #[test]
    fn rest_create_args_encode_labels_as_array_fields() {
        let backend = GitHubBackend::new(Some("owner/repo".to_string()));
        let labels = vec!["type:enhancement".to_string(), "phase:created".to_string()];

        let args = backend.rest_create_args("health gate", "body", &labels);

        assert_eq!(
            args,
            vec![
                "api",
                "-X",
                "POST",
                "repos/owner/repo/issues",
                "-f",
                "title=health gate",
                "-f",
                "body=body",
                "-f",
                "labels[]=type:enhancement",
                "-f",
                "labels[]=phase:created",
            ]
        );
    }
}
// CODEGEN-END
