//! `IssueBackend` trait — uniform interface over local files, GitHub, GitLab, Jira.
//!
//! Score's CLI (and eventually agents) talk to this trait. Backend selection
//! happens in `crate::issues::factory::make_backend` based on
//! `SddConfig.issue_platform.type`.

use super::types::{Issue, IssueFilter, IssuePatch};
use anyhow::Result;
use async_trait::async_trait;

/// A storage backend for issue artifacts.
///
/// Methods are async because GitHub/GitLab/Jira backends shell out or do
/// network IO. The local filesystem backend is sync-under-the-hood but
/// still exposes the async surface for interface parity.
#[async_trait]
pub trait IssueBackend: Send + Sync {
    /// Human-readable backend name (`"local"`, `"github"`, `"gitlab"`, `"jira"`).
    fn name(&self) -> &'static str;

    /// Whether `write()` is supported. GitHub/GitLab backends are currently
    /// read-only in the MVP and should return `false`.
    fn is_writable(&self) -> bool {
        true
    }

    /// List all issues matching the filter. Backends may apply the filter
    /// server-side (GitHub's `--label` flag) or client-side (fall through
    /// to `IssueFilter::matches`).
    async fn list(&self, filter: &IssueFilter) -> Result<Vec<Issue>>;

    /// Look up a single issue by its canonical local identifier.
    ///
    /// `id` can be:
    /// - a slug (e.g. `"enhancement-issue-authoring-notation"`) — local backend primary
    /// - a numeric string (e.g. `"1179"`) — GitHub/GitLab primary
    ///
    /// Backends should handle both forms when possible.
    async fn get(&self, id: &str) -> Result<Option<Issue>>;

    /// Persist an issue. For local backend, this writes the .md file.
    /// For GitHub/GitLab, this would POST to the API (not implemented in
    /// MVP — returns `Err(unsupported)`).
    async fn write(&self, issue: &Issue) -> Result<()>;

    /// Create a new issue. Returns the created issue (with slug, id, url populated).
    ///
    /// Default: returns "not supported" error. Backends opt in by overriding.
    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R1 #R8
    async fn create(&self, issue: &Issue) -> Result<Issue> {
        let _ = issue;
        anyhow::bail!("{} backend does not support create", self.name())
    }

    /// Apply a partial update to an existing issue.
    ///
    /// Default: returns "not supported" error. Backends opt in by overriding.
    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R2 #R8
    async fn update(&self, id: &str, patch: &IssuePatch) -> Result<Issue> {
        let _ = (id, patch);
        anyhow::bail!("{} backend does not support update", self.name())
    }

    /// Close an issue, optionally with a reason comment.
    ///
    /// Default: returns "not supported" error. Backends opt in by overriding.
    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R3 #R8
    async fn close(&self, id: &str, reason: Option<&str>) -> Result<()> {
        let _ = (id, reason);
        anyhow::bail!("{} backend does not support close", self.name())
    }

    /// Search issues by a text query. Returns matching issues.
    ///
    /// Default: calls `list` with no filter then greps title+body client-side.
    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R4 #R8
    async fn search(&self, query: &str) -> Result<Vec<Issue>> {
        let all = self.list(&IssueFilter::default()).await?;
        let q = query.to_ascii_lowercase();
        Ok(all
            .into_iter()
            .filter(|i| {
                i.title.to_ascii_lowercase().contains(&q)
                    || i.body.to_ascii_lowercase().contains(&q)
            })
            .collect())
    }
}

/// Pull all issues matching `filter` from `source` and write them to `target`.
/// Existing issues in `target` are matched by `id` (GitHub number) — their
/// `slug` is preserved to avoid breaking hand-picked filenames.
///
/// Returns the number of issues written.
pub async fn sync(
    source: &dyn IssueBackend,
    target: &dyn IssueBackend,
    filter: &IssueFilter,
) -> Result<SyncReport> {
    if !target.is_writable() {
        anyhow::bail!(
            "sync target '{}' is read-only; cannot push issues to it",
            target.name()
        );
    }

    // Build a lookup of existing target issues keyed by platform id (github_id or gitlab_id),
    // so we can preserve hand-picked slugs on update.
    let existing = target.list(&IssueFilter::default()).await?;
    let mut slug_by_platform_id: std::collections::HashMap<u64, String> =
        std::collections::HashMap::new();
    for e in &existing {
        if let Some(gid) = e.github_id {
            slug_by_platform_id.insert(gid, e.slug.clone());
        }
        if let Some(gid) = e.gitlab_id {
            slug_by_platform_id.insert(gid, e.slug.clone());
        }
    }

    let source_issues = source.list(filter).await?;
    let mut written = 0;
    let mut created = 0;
    let mut updated = 0;

    for mut issue in source_issues {
        let platform_id = issue.github_id.or(issue.gitlab_id);
        if let Some(pid) = platform_id {
            if let Some(existing_slug) = slug_by_platform_id.get(&pid) {
                // Preserve hand-picked slug for existing issues
                issue.slug = existing_slug.clone();
                updated += 1;
            } else {
                // New tracker-backed issue file — key by the native tracker id.
                issue.slug = pid.to_string();
                created += 1;
            }
        } else {
            issue.slug = issue.default_slug();
            created += 1;
        }

        target.write(&issue).await?;
        written += 1;
    }

    Ok(SyncReport {
        fetched: written,
        created,
        updated,
    })
}

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/backend.md#schema
// CODEGEN-BEGIN
/// Summary of an issue sync operation.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/backend.md#schema
#[derive(Debug, Clone, Copy)]
pub struct SyncReport {
    /// Number of issues fetched from source.
    pub fetched: usize,
    /// Number of issues newly created on target.
    pub created: usize,
    /// Number of issues updated on target.
    pub updated: usize,
}
// CODEGEN-END
