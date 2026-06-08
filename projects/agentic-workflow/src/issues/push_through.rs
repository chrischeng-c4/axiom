// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/logic/issues/push_through_source.md#source
// CODEGEN-BEGIN

//! `push_through` — write-through helper that powers the `validate` step
//! in the Phase C remote-as-source pipeline.
//!
//! Contract:
//!
//! 1. Read the local lifecycle issue file at `issue_path` and parse it back into an
//!    `Issue`.
//! 2. Call `backend.write(issue)` to push the merged state to the
//!    configured remote (gh / glab).
//! 3. Call `backend.get(slug)` to refresh the issue from remote (so any
//!    server-side normalisation — label ordering, body whitespace,
//!    timestamps — round-trips back into the lifecycle issue file).
//! 4. Re-serialise the refreshed issue and overwrite that file.
//!
//! On any failure the issue file is left untouched at its post-merge
//! state; the caller is responsible for `git checkout -- <issue_path>` to
//! roll back, and for emitting an error envelope.
//!
//! `LocalBackend` is a no-op for steps 2–3 because the lifecycle file IS
//! the storage; calling `push_through` against it is safe and just rewrites
//! the same bytes.

use crate::issues::backend::IssueBackend;
use crate::issues::types::Issue;
use anyhow::{Context, Result};
use std::path::Path;

/// Push the local lifecycle issue file at `issue_path` through `backend`,
/// refresh it from the remote, and rewrite the file.
///
/// Returns the post-refresh `Issue` so the caller can inspect any
/// server-side normalisation (e.g. canonicalised label ordering) before
/// committing the lifecycle trailer.
/// @spec projects/agentic-workflow/tech-design/core/logic/issues/push_through_source.md#source
pub async fn push_through(
    issue_path: &Path,
    backend: &dyn IssueBackend,
    slug: &str,
) -> Result<Issue> {
    let raw = std::fs::read_to_string(issue_path)
        .with_context(|| format!("read issue file {}", issue_path.display()))?;
    let mut issue: Issue = parse_issue_file(&raw)
        .with_context(|| format!("parse issue file {}", issue_path.display()))?;
    // Slug carries `#[serde(skip)]` (it lives in the filename, not the
    // frontmatter), so parse_issue_file returns slug="". Restore it from the
    // function parameter before handing the issue to the backend.
    issue.slug = slug.to_string();

    // Canonical identity on a tracker backend is the platform-assigned
    // numeric id, not the kebab slug. Once `create()` populates
    // `issue.github_id` / `issue.gitlab_id` and that value is persisted to
    // the lifecycle issue file, every subsequent push_through must look the
    // issue up by that id. Falling back to the kebab slug here is only correct
    // for the very first push (the issue file has no id yet) — the prior implementation
    // looked up by slug unconditionally, and because `encode_labels` no
    // longer emits a `slug:<kebab>` label, `resolve_slug` always returned
    // None on subsequent pushes, causing the existence probe to miss and
    // re-route to `create()`. Result: a fresh GitHub issue per CRRR phase
    // transition (the 7-dupe chain on aw wi sprint).
    let lookup_id: String = issue
        .github_id
        .map(|n| n.to_string())
        .or_else(|| issue.gitlab_id.map(|n| n.to_string()))
        .unwrap_or_else(|| slug.to_string());

    let exists_remotely = backend
        .get(&lookup_id)
        .await
        .with_context(|| format!("probe remote for issue '{}' before push", lookup_id))?
        .is_some();

    let refreshed = if exists_remotely {
        backend
            .write(&issue)
            .await
            .with_context(|| format!("update issue '{}' on remote backend", lookup_id))?;
        backend
            .get(&lookup_id)
            .await
            .with_context(|| format!("refresh issue '{}' from remote backend", lookup_id))?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "remote backend lost issue '{}' immediately after update",
                    lookup_id
                )
            })?
    } else {
        backend
            .create(&issue)
            .await
            .with_context(|| format!("create issue '{}' on remote backend", lookup_id))?
    };

    let serialised = serialise_issue_file(&refreshed)?;
    std::fs::write(issue_path, serialised)
        .with_context(|| format!("rewrite issue file {}", issue_path.display()))?;

    Ok(refreshed)
}

/// Parse a lifecycle issue file back into an `Issue`. Splits frontmatter (YAML) from
/// the body markdown.
fn parse_issue_file(raw: &str) -> Result<Issue> {
    let stripped = raw.strip_prefix("---\n").ok_or_else(|| {
        anyhow::anyhow!("issue file does not start with `---\\n` frontmatter delimiter")
    })?;
    let end = stripped
        .find("\n---\n")
        .ok_or_else(|| anyhow::anyhow!("issue file frontmatter missing closing `\\n---\\n`"))?;
    let frontmatter = &stripped[..end];
    let body = &stripped[end + 5..];

    let mut issue: Issue =
        serde_yaml::from_str(frontmatter).context("parse issue frontmatter as Issue")?;
    issue.body = body.trim_end_matches('\n').to_string();
    Ok(issue)
}

/// Serialise an `Issue` back into the lifecycle issue-file shape: YAML frontmatter
/// plus body, separated by `---` delimiters.
fn serialise_issue_file(issue: &Issue) -> Result<String> {
    let mut clone = issue.clone();
    let body = std::mem::take(&mut clone.body);
    let frontmatter =
        serde_yaml::to_string(&clone).context("serialise Issue frontmatter as YAML")?;
    Ok(format!("---\n{}---\n{}\n", frontmatter, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issues::backend::IssueBackend;
    use crate::issues::types::{IssueFilter, IssueState, IssueType};
    use async_trait::async_trait;
    use std::sync::Mutex;

    fn fixture_issue() -> Issue {
        Issue {
            issue_type: IssueType::Bug,
            title: "round-trip me".into(),
            state: IssueState::Open,
            id: None,
            github_id: None,
            gitlab_id: None,
            url: None,
            author: None,
            labels: vec!["crate:agentic-workflow".into()],
            created_at: None,
            updated_at: None,
            slug: "round-trip".into(),
            body: "Body line 1\n\nBody line 2".into(),
            related: vec![],
            implements: vec![],
            phase: Some("td_inited".into()),
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
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        }
    }

    #[test]
    fn issue_file_round_trip_preserves_body_and_frontmatter() {
        let issue = fixture_issue();
        let serialised = serialise_issue_file(&issue).unwrap();
        let parsed = parse_issue_file(&serialised).unwrap();
        assert_eq!(parsed.title, "round-trip me");
        assert_eq!(parsed.body, "Body line 1\n\nBody line 2");
        assert_eq!(parsed.labels, vec!["crate:agentic-workflow".to_string()]);
        assert_eq!(parsed.phase.as_deref(), Some("td_inited"));
    }

    /// Fake backend that mimics GitHub's identity model: `create()` assigns
    /// a numeric id; that id is the only key `get()` recognises on
    /// subsequent calls. A kebab slug never resolves (mirroring the real
    /// GitHub backend after `encode_labels` stopped emitting `slug:*`
    /// labels).
    #[derive(Default)]
    struct FakeRemote {
        inner: Mutex<FakeRemoteInner>,
    }

    #[derive(Default)]
    struct FakeRemoteInner {
        store: Vec<Issue>,
        next_id: u64,
        create_calls: u32,
        write_calls: u32,
    }

    #[async_trait]
    impl IssueBackend for FakeRemote {
        fn name(&self) -> &'static str {
            "fake-remote"
        }

        async fn list(&self, _filter: &IssueFilter) -> Result<Vec<Issue>> {
            Ok(self.inner.lock().unwrap().store.clone())
        }

        async fn get(&self, id: &str) -> Result<Option<Issue>> {
            let inner = self.inner.lock().unwrap();
            let Ok(n) = id.parse::<u64>() else {
                // Mimics GitHub: kebab slug never resolves once the
                // `slug:*` label is gone.
                return Ok(None);
            };
            Ok(inner.store.iter().find(|i| i.github_id == Some(n)).cloned())
        }

        async fn write(&self, issue: &Issue) -> Result<()> {
            let mut inner = self.inner.lock().unwrap();
            inner.write_calls += 1;
            let Some(n) = issue.github_id else {
                anyhow::bail!("write() called with no github_id");
            };
            let target = inner
                .store
                .iter_mut()
                .find(|i| i.github_id == Some(n))
                .ok_or_else(|| anyhow::anyhow!("write() of unknown id {}", n))?;
            *target = issue.clone();
            Ok(())
        }

        async fn create(&self, issue: &Issue) -> Result<Issue> {
            let mut inner = self.inner.lock().unwrap();
            inner.create_calls += 1;
            inner.next_id += 1;
            let n = inner.next_id;
            let mut staged = issue.clone();
            staged.github_id = Some(n);
            staged.slug = n.to_string();
            inner.store.push(staged.clone());
            Ok(staged)
        }
    }

    /// Regression: prior to the fix, `push_through` always looked up the
    /// remote by the kebab `slug` argument. Because no `slug:<kebab>`
    /// label is emitted by `encode_labels`, GitHub's `resolve_slug`
    /// always returned None, so the second push fell through to
    /// `backend.create()` and minted a fresh issue — the root cause of
    /// the 7-dupe chain on the aw wi sprint feature.
    #[tokio::test]
    async fn push_through_uses_github_id_after_first_create() {
        let issue = fixture_issue();
        let serialised = serialise_issue_file(&issue).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let issue_file = dir.path().join("round-trip.md");
        std::fs::write(&issue_file, serialised).unwrap();

        let backend = FakeRemote::default();

        // First push: issue file has no github_id, falls through to create.
        let first = push_through(&issue_file, &backend, "round-trip")
            .await
            .unwrap();
        assert_eq!(first.github_id, Some(1));

        // Second push: issue file now persists github_id=1. Must update the
        // same issue, not create a second one.
        let second = push_through(&issue_file, &backend, "round-trip")
            .await
            .unwrap();
        assert_eq!(second.github_id, Some(1));

        let inner = backend.inner.lock().unwrap();
        assert_eq!(
            inner.create_calls, 1,
            "second push_through must NOT create a fresh issue"
        );
        assert_eq!(inner.write_calls, 1, "second push must write-update");
        assert_eq!(inner.store.len(), 1, "remote must hold exactly one issue");
    }
}

// CODEGEN-END
