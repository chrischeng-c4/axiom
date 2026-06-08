// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/backends/gitlab_preamble_source.md#source
// CODEGEN-BEGIN
//! GitLab backend -- shells out to the `glab` CLI.
//!
//! The CRRR write contract round-trips through GitLab's native attributes
//! (title, state, description, labels) — see `crate::issues::labels` for
//! the label-prefix scheme that encodes the rest of `Issue`'s CRRR state.
//! `slug:*` labels are treated as legacy aliases; the GitLab issue iid is
//! the canonical identity.
//!
//! Authentication is delegated to `glab auth login`. Self-hosted hosts go
//! through the `GITLAB_HOST` environment variable.

// @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R6

use crate::issues::backend::IssueBackend;
use crate::issues::labels;
use crate::issues::types::{Issue, IssueFilter, IssuePatch, IssueState, IssueType};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::process::Command;
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/backends/gitlab.md#schema
// CODEGEN-BEGIN
/// Issue backend that calls the `glab` CLI.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/backends/gitlab.md#schema
pub struct GitLabBackend {
    /// Optional `owner/repo` slug. None = use CWD-detected repo.
    repo: Option<String>,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/backends/gitlab_runtime_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/backends/gitlab_runtime_source.md#source
impl GitLabBackend {
    pub fn new(repo: Option<String>) -> Self {
        Self { repo }
    }

    /// Construct with optional self-hosted host (e.g. `gitlab.example.com`).
    /// `glab` reads `GITLAB_HOST` to route requests; we set it here so the
    /// existing `glab_repo_args` plumbing is unchanged.
    pub fn with_host(repo: Option<String>, host: Option<String>) -> Self {
        if let Some(h) = host.as_deref() {
            std::env::set_var("GITLAB_HOST", h);
        }
        Self { repo }
    }

    fn glab_repo_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(r) = &self.repo {
            args.push("--repo".into());
            args.push(r.clone());
        }
        args
    }

    /// Resolve a legacy slug alias to a GitLab issue iid by querying the
    /// `slug:<slug>` label.
    fn resolve_slug(&self, slug: &str) -> Result<Option<u64>> {
        let mut args = vec![
            "issue".to_string(),
            "list".to_string(),
            "--label".to_string(),
            format!("{}{}", labels::SLUG_PREFIX, slug),
            "--state".to_string(),
            "all".to_string(),
            "--output".to_string(),
            "json".to_string(),
        ];
        args.extend(self.glab_repo_args());

        let output = run_glab(&args)?;
        let items: Vec<Value> = serde_json::from_str(&output)
            .with_context(|| format!("failed to parse glab slug-resolve JSON: {}", output))?;
        if items.is_empty() {
            return Ok(None);
        }
        let iid = items
            .iter()
            .filter_map(|v| v["iid"].as_u64())
            .min()
            .ok_or_else(|| anyhow!("slug:{slug} matched but no iid field"))?;
        Ok(Some(iid))
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/backends/gitlab_runtime_source.md#source
impl IssueBackend for GitLabBackend {
    fn name(&self) -> &'static str {
        "gitlab"
    }

    fn is_writable(&self) -> bool {
        true
    }

    async fn list(&self, filter: &IssueFilter) -> Result<Vec<Issue>> {
        let mut args = vec![
            "issue".to_string(),
            "list".to_string(),
            "--output".to_string(),
            "json".to_string(),
        ];

        // Server-side filters
        match filter.state {
            Some(IssueState::Open) => {
                args.push("--state".into());
                args.push("opened".into());
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

        args.extend(self.glab_repo_args());

        let output = run_glab(&args)?;
        let items: Vec<Value> =
            serde_json::from_str(&output).context("Failed to parse glab issue list JSON")?;

        let mut issues: Vec<Issue> = items.iter().map(parse_glab_issue).collect::<Result<_>>()?;

        // Client-side filters that glab doesn't support natively
        if filter.issue_type.is_some() || filter.author.is_some() {
            issues.retain(|i| filter.matches(i));
        }

        issues.sort_by_key(|i| i.gitlab_id.unwrap_or(0));
        Ok(issues)
    }

    /// Look up an issue by numeric GitLab iid OR legacy slug alias.
    async fn get(&self, id: &str) -> Result<Option<Issue>> {
        let iid: u64 = match id.parse() {
            Ok(n) => n,
            Err(_) => match self.resolve_slug(id)? {
                Some(n) => n,
                None => return Ok(None),
            },
        };

        let mut args = vec![
            "issue".to_string(),
            "view".to_string(),
            iid.to_string(),
            "--output".to_string(),
            "json".to_string(),
        ];
        args.extend(self.glab_repo_args());

        let output = match run_glab(&args) {
            Ok(s) => s,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("not found") || msg.contains("404") {
                    return Ok(None);
                }
                return Err(e);
            }
        };

        let json: Value =
            serde_json::from_str(&output).context("Failed to parse glab issue view JSON")?;
        Ok(Some(parse_glab_issue(&json)?))
    }

    async fn write(&self, issue: &Issue) -> Result<()> {
        let iid: u64 = match issue.gitlab_id {
            Some(n) => n,
            None => match issue.slug.parse::<u64>() {
                Ok(n) => n,
                Err(_) => self.resolve_slug(&issue.slug)?.ok_or_else(|| {
                    anyhow!(
                        "GitLabBackend::write: cannot locate issue (slug='{}', no gitlab_id)",
                        issue.slug
                    )
                })?,
            },
        };

        // Fetch current for diff.
        let mut view_args = vec![
            "issue".to_string(),
            "view".to_string(),
            iid.to_string(),
            "--output".to_string(),
            "json".to_string(),
        ];
        view_args.extend(self.glab_repo_args());
        let view_out = run_glab(&view_args)?;
        let view: Value = serde_json::from_str(&view_out)
            .with_context(|| format!("failed to parse glab view JSON: {}", view_out))?;

        let current_title = view["title"].as_str().unwrap_or("").to_string();
        let current_body = view["description"].as_str().unwrap_or("").to_string();
        let current_state_raw = view["state"].as_str().unwrap_or("opened").to_string();
        let current_labels: Vec<String> = view["labels"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|l| l.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let desired_labels = labels::encode_labels(issue);
        let (to_add, to_remove) = labels::diff_labels(&current_labels, &desired_labels);

        let title_changed = issue.title != current_title;
        let body_changed = issue.body != current_body;
        if title_changed || body_changed || !to_add.is_empty() || !to_remove.is_empty() {
            let mut args = vec!["issue".to_string(), "update".to_string(), iid.to_string()];
            if title_changed {
                args.push("--title".into());
                args.push(issue.title.clone());
            }
            if body_changed {
                args.push("--description".into());
                args.push(issue.body.clone());
            }
            for l in &to_add {
                args.push("--label".into());
                args.push(l.clone());
            }
            for l in &to_remove {
                args.push("--unlabel".into());
                args.push(l.clone());
            }
            args.extend(self.glab_repo_args());
            run_glab(&args)?;
        }

        // State transitions: glab uses `issue close` / `issue reopen`.
        let desired_open = matches!(issue.state, IssueState::Open | IssueState::Draft);
        let currently_open = current_state_raw == "opened";
        if desired_open && !currently_open {
            let mut args = vec!["issue".to_string(), "reopen".to_string(), iid.to_string()];
            args.extend(self.glab_repo_args());
            run_glab(&args)?;
        } else if !desired_open && currently_open {
            let mut args = vec!["issue".to_string(), "close".to_string(), iid.to_string()];
            args.extend(self.glab_repo_args());
            run_glab(&args)?;
        }

        Ok(())
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R1
    async fn create(&self, issue: &Issue) -> Result<Issue> {
        let mut staged = issue.clone();
        staged.slug.clear();
        let desired = labels::encode_labels(&staged);

        let mut args = vec![
            "issue".to_string(),
            "create".to_string(),
            "--title".to_string(),
            staged.title.clone(),
            "--description".to_string(),
            staged.body.clone(),
        ];

        if !desired.is_empty() {
            args.push("--label".into());
            args.push(desired.join(","));
        }

        args.push("--output".into());
        args.push("json".into());
        args.extend(self.glab_repo_args());

        let output = run_glab(&args)?;
        let json: Value =
            serde_json::from_str(&output).context("Failed to parse glab issue create JSON")?;

        let iid = json["iid"].as_u64();
        let url = json["web_url"].as_str().map(String::from);

        staged.gitlab_id = iid;
        staged.url = url;
        staged.state = IssueState::Open;
        if let Some(n) = iid {
            staged.slug = n.to_string();
            self.write(&staged).await?;
        }
        Ok(staged)
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R2
    async fn update(&self, id: &str, patch: &IssuePatch) -> Result<Issue> {
        let iid: u64 = match id.parse::<u64>() {
            Ok(n) => n,
            Err(_) => self
                .resolve_slug(id)?
                .ok_or_else(|| anyhow!("issue '{}' not found on GitLab", id))?,
        };

        let mut current = self
            .get(&iid.to_string())
            .await?
            .ok_or_else(|| anyhow!("issue !{} not found", iid))?;
        patch.apply(&mut current);
        self.write(&current).await?;

        self.get(&iid.to_string())
            .await?
            .ok_or_else(|| anyhow!("issue !{} not found after update", iid))
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R3
    async fn close(&self, id: &str, reason: Option<&str>) -> Result<()> {
        let iid: u64 = match id.parse::<u64>() {
            Ok(n) => n,
            Err(_) => self
                .resolve_slug(id)?
                .ok_or_else(|| anyhow!("issue '{}' not found on GitLab", id))?,
        };

        let mut args = vec!["issue".to_string(), "close".to_string(), iid.to_string()];
        args.extend(self.glab_repo_args());
        run_glab(&args)?;

        // Post comment for the reason if provided
        if let Some(r) = reason {
            let mut note_args = vec![
                "issue".to_string(),
                "note".to_string(),
                iid.to_string(),
                "--message".to_string(),
                r.to_string(),
            ];
            note_args.extend(self.glab_repo_args());
            // Best-effort: don't fail close if the comment fails
            let _ = run_glab(&note_args);
        }

        Ok(())
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R4
    async fn search(&self, query: &str) -> Result<Vec<Issue>> {
        let mut args = vec![
            "issue".to_string(),
            "list".to_string(),
            "--search".to_string(),
            query.to_string(),
            "--output".to_string(),
            "json".to_string(),
            "--state".to_string(),
            "all".to_string(),
        ];
        args.extend(self.glab_repo_args());

        let output = run_glab(&args)?;
        let items: Vec<Value> =
            serde_json::from_str(&output).context("Failed to parse glab search JSON")?;

        items.iter().map(parse_glab_issue).collect()
    }
}

/// Parse a single glab JSON issue object into our `Issue` type. The GitLab
/// iid is the canonical local slug; any `slug:*` label is decoded only as a
/// legacy lookup alias.
fn parse_glab_issue(v: &Value) -> Result<Issue> {
    let iid = v["iid"]
        .as_u64()
        .ok_or_else(|| anyhow!("glab issue missing iid: {}", v))?;
    let title = v["title"].as_str().unwrap_or("(untitled)").to_string();
    // glab uses "description" for body, not "body"
    let body = v["description"].as_str().unwrap_or("").to_string();

    let state_raw = v["state"].as_str().unwrap_or("opened");
    let state = match state_raw {
        "opened" => IssueState::Open,
        "closed" => IssueState::Closed,
        _ => IssueState::Open,
    };

    // glab returns labels as an array of strings (not objects)
    let raw_labels: Vec<String> = v["labels"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|l| l.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let issue_type = IssueType::from_labels(&raw_labels);
    let decoded = labels::decode_labels(&raw_labels);
    let canonical_slug = iid.to_string();

    let author = v["author"]["username"].as_str().map(String::from);
    let created_at = v["created_at"].as_str().map(String::from);
    let updated_at = v["updated_at"].as_str().map(String::from);
    let url = v["web_url"].as_str().map(String::from);

    Ok(Issue {
        issue_type,
        title,
        state,
        id: None,
        github_id: None,
        gitlab_id: Some(iid),
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

fn run_glab(args: &[String]) -> Result<String> {
    let output = Command::new("glab")
        .args(args)
        .output()
        .context("Failed to execute glab -- is the GitLab CLI installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("glab {:?} failed: {}", args, stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_glab_issue_uses_iid_as_canonical_slug() {
        let value = json!({
            "iid": 1887,
            "title": "bug(score): slug round trip",
            "state": "opened",
            "labels": [
                "type:bug",
                "phase:created",
                "slug:bug-slug-round-trip-broken-local-cache-slug-d"
            ],
            "author": {"username": "tester"},
            "created_at": "2026-05-11T00:00:00Z",
            "updated_at": "2026-05-11T00:00:00Z",
            "web_url": "https://gitlab.com/owner/repo/-/issues/1887",
            "description": "body"
        });

        let issue = parse_glab_issue(&value).unwrap();

        assert_eq!(issue.gitlab_id, Some(1887));
        assert_eq!(issue.slug, "1887");
        assert_eq!(issue.phase.as_deref(), Some("created"));
    }
}
// CODEGEN-END
