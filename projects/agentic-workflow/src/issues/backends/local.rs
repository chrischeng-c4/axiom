// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/backends/local_preamble_source.md#source
// CODEGEN-BEGIN
//! Local filesystem backend — reads and writes `{issues_dir}/{open,closed}/*.md`.
//!
//! Issues are physically separated into `open/` and `closed/` subdirectories,
//! mirroring GitHub/GitLab's two-state model. Each issue is a Markdown file
//! with YAML frontmatter. Project-root instances store lifecycle working
//! copies under `/tmp/aw/workspaces/<workspace>/issues`; remote read-through
//! cache instances live under `/tmp/aw/issues`. Tracker-backed issues use the
//! tracker-local number (`github_id` / `gitlab_id`) as their canonical file
//! key; legacy title slugs remain readable as aliases when they already exist
//! on disk.

use crate::issues::backend::IssueBackend;
use crate::issues::types::{Issue, IssueFilter, IssuePatch, IssueState, IssueType};
use crate::parser::frontmatter::parse_document;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/backends/local_helpers_source.md#source
// CODEGEN-BEGIN
/// State subdirectories within the issues root.
const STATE_SUBDIRS: [&str; 2] = ["open", "closed"];

/// Map `IssueState` to the subdirectory name.
fn subdir_for_state(state: IssueState) -> &'static str {
    match state {
        IssueState::Open | IssueState::Draft => "open",
        IssueState::Closed => "closed",
    }
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/backends/local.md#schema
// CODEGEN-BEGIN
/// Backend that stores issues as files under an issue directory.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/backends/local.md#schema
pub struct LocalBackend {
    /// Directory containing issue files.
    issues_dir: PathBuf,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/backends/local_runtime_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/backends/local_runtime_source.md#source
impl LocalBackend {
    /// Create a lifecycle backend rooted under `/tmp/aw`.
    pub fn from_project_root(project_root: &Path) -> Self {
        let issues_dir = crate::shared::workspace::issues_path(project_root);
        Self { issues_dir }
    }

    /// Create a backend at an explicit directory path. Mainly for tests.
    pub fn at(issues_dir: PathBuf) -> Self {
        Self { issues_dir }
    }

    /// Return the root directory backing this local issue store.
    pub fn issues_dir(&self) -> &Path {
        &self.issues_dir
    }

    /// Return the expected path for an issue's current state.
    pub fn issue_path(&self, issue: &Issue) -> PathBuf {
        let subdir = subdir_for_state(issue.state);
        self.issues_dir
            .join(subdir)
            .join(format!("{}.md", issue.slug))
    }

    /// Search `open/` then `closed/` for a file matching the given slug.
    fn find_issue_path(&self, slug: &str) -> Option<PathBuf> {
        let filename = format!("{}.md", slug);
        for subdir in &STATE_SUBDIRS {
            let path = self.issues_dir.join(subdir).join(&filename);
            if path.exists() {
                return Some(path);
            }
        }
        None
    }

    /// Search `open/` then `closed/` for a canonical id-prefixed issue file.
    ///
    /// This keeps older `<id>-<title-kebab>.md` files readable while the
    /// steady-state tracker key moves to `<id>.md`.
    fn find_issue_path_by_id_prefix(&self, id: u64) -> Result<Option<PathBuf>> {
        let prefix = format!("{id}-");
        let mut matches = Vec::new();
        for subdir in &STATE_SUBDIRS {
            let dir = self.issues_dir.join(subdir);
            if !dir.exists() {
                continue;
            }
            for entry in std::fs::read_dir(&dir)
                .with_context(|| format!("Failed to read {}", dir.display()))?
            {
                let path = entry?.path();
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if stem == id.to_string() || stem.starts_with(&prefix) {
                    matches.push(path);
                }
            }
        }
        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.pop()),
            _ => anyhow::bail!("multiple local issue files match tracker id {}", id),
        }
    }

    /// Load all issues from both `open/` and `closed/` subdirectories.
    fn load_all(&self) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        for subdir in &STATE_SUBDIRS {
            let dir = self.issues_dir.join(subdir);
            if !dir.exists() {
                continue;
            }
            for entry in std::fs::read_dir(&dir)
                .with_context(|| format!("Failed to read {}", dir.display()))?
            {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }
                // Skip hidden files like .gitkeep
                if path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with('.'))
                    .unwrap_or(false)
                {
                    continue;
                }

                let issue = parse_issue_file(&path)?;
                issues.push(issue);
            }
        }

        // Deterministic order: by slug (= filename stem).
        issues.sort_by(|a, b| a.slug.cmp(&b.slug));
        Ok(issues)
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/backends/local_runtime_source.md#source
impl IssueBackend for LocalBackend {
    fn name(&self) -> &'static str {
        "local"
    }

    async fn list(&self, filter: &IssueFilter) -> Result<Vec<Issue>> {
        let all = self.load_all()?;
        Ok(all.into_iter().filter(|i| filter.matches(i)).collect())
    }

    async fn get(&self, id: &str) -> Result<Option<Issue>> {
        // Try slug first — search both open/ and closed/.
        if let Some(path) = self.find_issue_path(id) {
            return Ok(Some(parse_issue_file(&path)?));
        }

        // Fallback: match by numeric github_id or gitlab_id in frontmatter.
        if let Ok(num) = id.parse::<u64>() {
            if let Some(path) = self.find_issue_path_by_id_prefix(num)? {
                return Ok(Some(parse_issue_file(&path)?));
            }
            for issue in self.load_all()? {
                if issue.github_id == Some(num) || issue.gitlab_id == Some(num) {
                    return Ok(Some(issue));
                }
            }
        }

        Ok(None)
    }

    async fn write(&self, issue: &Issue) -> Result<()> {
        if issue.slug.is_empty() {
            anyhow::bail!("cannot write issue with empty slug");
        }

        let subdir = subdir_for_state(issue.state);
        let target_dir = self.issues_dir.join(subdir);
        std::fs::create_dir_all(&target_dir)
            .with_context(|| format!("Failed to create {}", target_dir.display()))?;

        let filename = format!("{}.md", issue.slug);
        let target_path = target_dir.join(&filename);

        // Remove from the opposite subdirectory if it exists there
        let opposite = if subdir == "open" { "closed" } else { "open" };
        let old_path = self.issues_dir.join(opposite).join(&filename);
        if old_path.exists() {
            std::fs::remove_file(&old_path)
                .with_context(|| format!("Failed to remove old file {}", old_path.display()))?;
        }

        let mut stamped = issue.clone();
        let now = chrono::Utc::now().to_rfc3339();
        if stamped.created_at.is_none() {
            stamped.created_at = Some(now.clone());
        }
        stamped.updated_at = Some(now);

        let content = serialize_issue(&stamped)?;
        std::fs::write(&target_path, content)
            .with_context(|| format!("Failed to write {}", target_path.display()))?;
        Ok(())
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R1
    async fn create(&self, issue: &Issue) -> Result<Issue> {
        let mut created = issue.clone();
        if created.slug.is_empty() {
            created.slug = created.default_slug();
        }
        // Assign UUID if not already set
        if created.id.is_none() {
            created.id = Some(uuid::Uuid::new_v4().to_string());
        }
        if created.state == IssueState::Open
            && created.github_id.is_none()
            && created.gitlab_id.is_none()
        {
            // Local-only drafts start in draft state
            created.state = IssueState::Draft;
        }
        self.write(&created).await?;
        Ok(created)
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R2
    async fn update(&self, id: &str, patch: &IssuePatch) -> Result<Issue> {
        let mut issue = self
            .get(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", id))?;
        patch.apply(&mut issue);
        self.write(&issue).await?;
        Ok(issue)
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R3
    async fn close(&self, id: &str, _reason: Option<&str>) -> Result<()> {
        let mut issue = self
            .get(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", id))?;
        issue.state = IssueState::Closed;
        self.write(&issue).await?;
        Ok(())
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R4
    async fn search(&self, query: &str) -> Result<Vec<Issue>> {
        let all = self.load_all()?;
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
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/backends/local_frontmatter_source.md#source
// CODEGEN-BEGIN
/// Frontmatter representation — mirrors `Issue` but only the serializable parts.
/// We use an intermediate type so `Issue.slug` and `Issue.body` (which are
/// derived from file location / body text, not frontmatter) stay `#[serde(skip)]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IssueFrontmatter {
    #[serde(rename = "type")]
    issue_type: IssueType,
    title: String,
    state: IssueState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    github_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    gitlab_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    updated_at: Option<String>,
    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R5
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    related: Vec<String>,
    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R5
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    implements: Vec<String>,
    // REQ: REQ-001 — Issue-centric SDD workflow fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    /// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/issue-merge-target.md#schema
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    git_workflow: Option<String>,
    // REQ: R1 — Transient SDD fields (absorb STATE.yaml)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    change_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    iteration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    current_task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    impl_spec_phase: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    task_revisions: Option<HashMap<String, u32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    revision_counts: Option<HashMap<String, u32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    // REQ: R4 — CRR validation errors
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    validation_errors: Vec<String>,
    // REQ: issue-crrr-state-machine#R10 — CRRR review counter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    review_count: Option<u8>,
    // REQ: issue-crrr-state-machine#R8, R9 — sections flagged by last review
    #[serde(default, skip_serializing_if = "Option::is_none")]
    flagged_sections: Option<Vec<crate::issues::types::IssueSection>>,
    // REQ: fill-loop-retry-cap
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fill_retry_count: Option<u8>,
    // @spec projects/agentic-workflow/tech-design/surface/specs/score-td-validate-lifecycle-extension.md#schema
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ship_status: Option<crate::issues::types::ShipStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ship_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    regen_verified_at: Option<String>,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/backends/local_persistence_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/backends/local_persistence_source.md#source
impl From<&Issue> for IssueFrontmatter {
    fn from(i: &Issue) -> Self {
        IssueFrontmatter {
            issue_type: i.issue_type,
            title: i.title.clone(),
            state: i.state,
            id: i.id.clone(),
            github_id: i.github_id,
            gitlab_id: i.gitlab_id,
            url: i.url.clone(),
            author: i.author.clone(),
            labels: i.labels.clone(),
            created_at: i.created_at.clone(),
            updated_at: i.updated_at.clone(),
            related: i.related.clone(),
            implements: i.implements.clone(),
            phase: i.phase.clone(),
            branch: i.branch.clone(),
            target_branch: i.target_branch.clone(),
            git_workflow: i.git_workflow.clone(),
            change_id: i.change_id.clone(),
            iteration: i.iteration,
            current_task_id: i.current_task_id.clone(),
            impl_spec_phase: i.impl_spec_phase.clone(),
            task_revisions: i.task_revisions.clone(),
            revision_counts: i.revision_counts.clone(),
            last_action: i.last_action.clone(),
            session_id: i.session_id.clone(),
            validation_errors: i.validation_errors.clone(),
            review_count: i.review_count,
            flagged_sections: i.flagged_sections.clone(),
            fill_retry_count: i.fill_retry_count,
            ship_status: i.ship_status,
            ship_commit: i.ship_commit.clone(),
            regen_verified_at: i.regen_verified_at.clone(),
        }
    }
}

fn parse_issue_file(path: &Path) -> Result<Issue> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let doc = parse_document::<IssueFrontmatter>(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    let slug = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();

    Ok(Issue {
        issue_type: doc.frontmatter.issue_type,
        title: doc.frontmatter.title,
        state: doc.frontmatter.state,
        id: doc.frontmatter.id,
        github_id: doc.frontmatter.github_id,
        gitlab_id: doc.frontmatter.gitlab_id,
        url: doc.frontmatter.url,
        author: doc.frontmatter.author,
        labels: doc.frontmatter.labels,
        created_at: doc.frontmatter.created_at,
        updated_at: doc.frontmatter.updated_at,
        slug,
        body: doc.body,
        related: doc.frontmatter.related,
        implements: doc.frontmatter.implements,
        phase: doc.frontmatter.phase,
        branch: doc.frontmatter.branch,
        target_branch: doc.frontmatter.target_branch,
        git_workflow: doc.frontmatter.git_workflow,
        change_id: doc.frontmatter.change_id,
        iteration: doc.frontmatter.iteration,
        current_task_id: doc.frontmatter.current_task_id,
        impl_spec_phase: doc.frontmatter.impl_spec_phase,
        task_revisions: doc.frontmatter.task_revisions,
        revision_counts: doc.frontmatter.revision_counts,
        last_action: doc.frontmatter.last_action,
        session_id: doc.frontmatter.session_id,
        validation_errors: doc.frontmatter.validation_errors,
        review_count: doc.frontmatter.review_count,
        flagged_sections: doc.frontmatter.flagged_sections,
        fill_retry_count: doc.frontmatter.fill_retry_count,
        ship_status: doc.frontmatter.ship_status,
        ship_commit: doc.frontmatter.ship_commit,
        regen_verified_at: doc.frontmatter.regen_verified_at,
    })
}

fn serialize_issue(issue: &Issue) -> Result<String> {
    let fm = IssueFrontmatter::from(issue);
    let fm_yaml = serde_yaml::to_string(&fm).context("Failed to serialize frontmatter")?;

    let mut out = String::with_capacity(fm_yaml.len() + issue.body.len() + 16);
    out.push_str("---\n");
    out.push_str(&fm_yaml);
    out.push_str("---\n\n");
    out.push_str(issue.body.trim_end());
    out.push('\n');
    Ok(out)
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/backends/local_tests_source.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_issue(slug: &str, github_id: Option<u64>) -> Issue {
        Issue {
            issue_type: IssueType::Enhancement,
            title: format!("test {}", slug),
            state: IssueState::Open,
            id: Some(uuid::Uuid::new_v4().to_string()),
            github_id,
            gitlab_id: None,
            url: None,
            author: Some("tester".into()),
            labels: vec!["type:enhancement".into(), "priority:p1".into()],
            created_at: None,
            updated_at: None,
            slug: slug.to_string(),
            body: "# Test\n\nBody content.".into(),
            related: vec![],
            implements: vec![],
            phase: None,
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

    #[tokio::test]
    async fn write_then_read_round_trip() {
        let tmp = TempDir::new().unwrap();
        let backend = LocalBackend::at(tmp.path().to_path_buf());

        let issue = make_issue("enhancement-test-foo", Some(42));
        backend.write(&issue).await.unwrap();

        let loaded = backend.get("enhancement-test-foo").await.unwrap().unwrap();
        assert_eq!(loaded.slug, "enhancement-test-foo");
        assert_eq!(loaded.title, "test enhancement-test-foo");
        assert_eq!(loaded.github_id, Some(42));
        assert_eq!(loaded.issue_type, IssueType::Enhancement);
        assert!(loaded.body.contains("Body content"));
    }

    #[tokio::test]
    async fn list_filters() {
        let tmp = TempDir::new().unwrap();
        let backend = LocalBackend::at(tmp.path().to_path_buf());

        let mut bug = make_issue("bug-one", Some(1));
        bug.issue_type = IssueType::Bug;
        let mut closed_bug = make_issue("bug-two", Some(2));
        closed_bug.issue_type = IssueType::Bug;
        closed_bug.state = IssueState::Closed;
        let epic = {
            let mut i = make_issue("epic-three", Some(3));
            i.issue_type = IssueType::Epic;
            i
        };

        backend.write(&bug).await.unwrap();
        backend.write(&closed_bug).await.unwrap();
        backend.write(&epic).await.unwrap();

        let all = backend.list(&IssueFilter::default()).await.unwrap();
        assert_eq!(all.len(), 3);

        let open_only = backend
            .list(&IssueFilter {
                state: Some(IssueState::Open),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(open_only.len(), 2);

        let bugs_only = backend
            .list(&IssueFilter {
                issue_type: Some(IssueType::Bug),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(bugs_only.len(), 2);
    }

    #[tokio::test]
    async fn get_by_numeric_id() {
        let tmp = TempDir::new().unwrap();
        let backend = LocalBackend::at(tmp.path().to_path_buf());
        backend
            .write(&make_issue("bug-alpha", Some(100)))
            .await
            .unwrap();

        let by_num = backend.get("100").await.unwrap();
        assert!(by_num.is_some());
        assert_eq!(by_num.unwrap().slug, "bug-alpha");

        let missing = backend.get("999").await.unwrap();
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn get_by_numeric_id_accepts_legacy_id_prefixed_filename() {
        let tmp = TempDir::new().unwrap();
        let backend = LocalBackend::at(tmp.path().to_path_buf());
        backend
            .write(&make_issue("1887-legacy-title-tail", Some(1887)))
            .await
            .unwrap();

        let by_num = backend.get("1887").await.unwrap().unwrap();
        assert_eq!(by_num.slug, "1887-legacy-title-tail");
        assert_eq!(by_num.github_id, Some(1887));
    }

    #[tokio::test]
    async fn write_requires_slug() {
        let tmp = TempDir::new().unwrap();
        let backend = LocalBackend::at(tmp.path().to_path_buf());
        let mut issue = make_issue("", None);
        issue.slug = String::new();
        let err = backend.write(&issue).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn write_routes_to_open_subdir() {
        let tmp = TempDir::new().unwrap();
        let backend = LocalBackend::at(tmp.path().to_path_buf());
        let issue = make_issue("bug-alpha", Some(1));
        backend.write(&issue).await.unwrap();
        assert!(tmp.path().join("open/bug-alpha.md").exists());
        assert!(!tmp.path().join("closed/bug-alpha.md").exists());
    }

    #[tokio::test]
    async fn close_moves_file_to_closed_subdir() {
        let tmp = TempDir::new().unwrap();
        let backend = LocalBackend::at(tmp.path().to_path_buf());
        backend
            .write(&make_issue("bug-beta", Some(2)))
            .await
            .unwrap();
        assert!(tmp.path().join("open/bug-beta.md").exists());

        backend.close("bug-beta", None).await.unwrap();
        assert!(!tmp.path().join("open/bug-beta.md").exists());
        assert!(tmp.path().join("closed/bug-beta.md").exists());
    }

    #[tokio::test]
    async fn list_scans_both_subdirs() {
        let tmp = TempDir::new().unwrap();
        let backend = LocalBackend::at(tmp.path().to_path_buf());
        backend
            .write(&make_issue("bug-one", Some(1)))
            .await
            .unwrap();
        let mut closed = make_issue("bug-two", Some(2));
        closed.state = IssueState::Closed;
        backend.write(&closed).await.unwrap();

        let all = backend.list(&IssueFilter::default()).await.unwrap();
        assert_eq!(all.len(), 2);
        assert!(tmp.path().join("open/bug-one.md").exists());
        assert!(tmp.path().join("closed/bug-two.md").exists());
    }

    #[tokio::test]
    async fn write_auto_stamps_timestamps() {
        let tmp = TempDir::new().unwrap();
        let backend = LocalBackend::at(tmp.path().to_path_buf());

        let mut issue = make_issue("bug-stamp", Some(7));
        assert!(issue.created_at.is_none());
        assert!(issue.updated_at.is_none());

        backend.write(&issue).await.unwrap();
        let first = backend.get("bug-stamp").await.unwrap().unwrap();
        let first_updated = first.updated_at.clone().expect("updated_at stamped");
        let first_created = first.created_at.clone().expect("created_at stamped");
        assert_eq!(first_updated, first_created);

        // updated_at advances on second write; created_at sticks.
        issue.created_at = first.created_at.clone();
        issue.updated_at = first.updated_at.clone();
        std::thread::sleep(std::time::Duration::from_millis(10));
        backend.write(&issue).await.unwrap();
        let second = backend.get("bug-stamp").await.unwrap().unwrap();
        assert_eq!(second.created_at, Some(first_created));
        assert_ne!(second.updated_at, Some(first_updated));
    }

    #[tokio::test]
    async fn update_state_moves_file() {
        let tmp = TempDir::new().unwrap();
        let backend = LocalBackend::at(tmp.path().to_path_buf());
        backend
            .write(&make_issue("bug-gamma", Some(3)))
            .await
            .unwrap();
        assert!(tmp.path().join("open/bug-gamma.md").exists());

        let patch = IssuePatch {
            state: Some(IssueState::Closed),
            ..Default::default()
        };
        backend.update("bug-gamma", &patch).await.unwrap();
        assert!(!tmp.path().join("open/bug-gamma.md").exists());
        assert!(tmp.path().join("closed/bug-gamma.md").exists());
    }

    // Regression for the IssuePatch::apply else-branch bug where review_count
    // and flagged_sections were silently dropped on patches that did not set
    // clear_transient. Two consecutive validate-style updates must persist the
    // counter, otherwise the CRRR R12 ceiling never trips.
    #[tokio::test]
    async fn review_count_round_trips_across_two_updates() {
        use crate::issues::types::IssueSection;

        let tmp = TempDir::new().unwrap();
        let backend = LocalBackend::at(tmp.path().to_path_buf());
        backend
            .write(&make_issue("enhancement-rev-count", Some(7)))
            .await
            .unwrap();

        let first = IssuePatch {
            phase: Some("reviewed".into()),
            review_count: Some(1),
            flagged_sections: Some(vec![IssueSection::Scope]),
            ..Default::default()
        };
        backend
            .update("enhancement-rev-count", &first)
            .await
            .unwrap();

        let after_first = backend.get("enhancement-rev-count").await.unwrap().unwrap();
        assert_eq!(after_first.review_count, Some(1));
        assert_eq!(
            after_first.flagged_sections,
            Some(vec![IssueSection::Scope])
        );
        assert_eq!(after_first.phase.as_deref(), Some("reviewed"));

        let second = IssuePatch {
            phase: Some("reviewed".into()),
            review_count: Some(2),
            flagged_sections: Some(vec![
                IssueSection::Requirements,
                IssueSection::ReferenceContext,
            ]),
            ..Default::default()
        };
        backend
            .update("enhancement-rev-count", &second)
            .await
            .unwrap();

        let after_second = backend.get("enhancement-rev-count").await.unwrap().unwrap();
        assert_eq!(after_second.review_count, Some(2));
        assert_eq!(
            after_second.flagged_sections,
            Some(vec![
                IssueSection::Requirements,
                IssueSection::ReferenceContext,
            ])
        );
    }

    // clear_transient must wipe review_count and flagged_sections back to None
    // — used by merge / reset paths to reset the CRRR counter cleanly.
    #[tokio::test]
    async fn clear_transient_wipes_review_count_and_flagged() {
        use crate::issues::types::IssueSection;

        let tmp = TempDir::new().unwrap();
        let backend = LocalBackend::at(tmp.path().to_path_buf());
        let mut seed = make_issue("enhancement-rev-clear", Some(8));
        seed.review_count = Some(2);
        seed.flagged_sections = Some(vec![IssueSection::Scope]);
        backend.write(&seed).await.unwrap();

        let patch = IssuePatch {
            clear_transient: true,
            ..Default::default()
        };
        backend
            .update("enhancement-rev-clear", &patch)
            .await
            .unwrap();

        let after = backend.get("enhancement-rev-clear").await.unwrap().unwrap();
        assert_eq!(after.review_count, None);
        assert_eq!(after.flagged_sections, None);
    }
}
// CODEGEN-END
