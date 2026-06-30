// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
// CODEGEN-BEGIN
//! Trait + impls for invoking the `aw` CLI from a Rust process.
//!
//! `aw wi` and `aw td` are CLI-only today (no Rust library API),
//! so the real impl shells out via `tokio::process::Command` and parses the
//! envelope JSON from stdout. A `MockScoreProcess` lets tests run without a real
//! `aw` binary.
//!
//! The trait takes `serde_json::Value` for fill-section args so the schema
//! stays in lockstep with the CLI's `Invoke.args` shape.

use crate::issues::{
    Issue as StoredIssue, IssueBackend as StoredIssueBackend, IssueFilter as StoredIssueFilter,
    IssueState as StoredIssueState, LocalBackend,
};
use crate::runtime::envelope::Envelope;
use crate::runtime::issue_backend::{
    BackendError, BackendKind, IssueBackend, IssueBody, IssueId, IssueRef,
    IssueState as RuntimeIssueState, ListFilter,
};
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::process::Command;

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#changes
#[derive(Debug, Error)]
pub enum ScoreProcessError {
    #[error("aw binary not found on PATH")]
    BinaryNotFound,
    #[error("aw exited non-zero ({code:?}): {stderr}")]
    NonZeroExit { code: Option<i32>, stderr: String },
    #[error("could not parse envelope JSON: {source}\n--- stdout ---\n{stdout}")]
    ParseEnvelope {
        source: serde_json::Error,
        stdout: String,
    },
    #[error("io error invoking aw: {0}")]
    Io(#[from] std::io::Error),
    #[error("mock has no canned response queued for `{0}`")]
    MockExhausted(String),
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub trait ScoreProcess: Send + Sync {
    async fn create(&self, title: &str) -> Result<Envelope, ScoreProcessError>;
    async fn fill_section_apply(
        &self,
        slug: &str,
        section: &str,
        body: &str,
    ) -> Result<Envelope, ScoreProcessError>;
    async fn review_apply(&self, slug: &str, body: &str) -> Result<Envelope, ScoreProcessError>;
    async fn validate(&self, slug: &str) -> Result<Envelope, ScoreProcessError>;
    async fn merge(&self, slug: &str) -> Result<Envelope, ScoreProcessError>;
}

/// Real impl — shells out to the `aw` binary on PATH.
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub struct RealScoreProcess {
    pub binary: String,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl Default for RealScoreProcess {
    fn default() -> Self {
        Self {
            binary: "aw".to_string(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl RealScoreProcess {
    pub fn new(binary: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
        }
    }

    async fn run(&self, args: &[&str]) -> Result<Envelope, ScoreProcessError> {
        let output = Command::new(&self.binary)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            return Err(ScoreProcessError::NonZeroExit {
                code: output.status.code(),
                stderr,
            });
        }

        // Score may print prelude lines before the JSON envelope on the last
        // line. Strategy: try the whole stdout, then walk back from the end
        // looking for a parseable JSON object.
        if let Ok(env) = serde_json::from_str::<Envelope>(stdout.trim()) {
            return Ok(env);
        }
        for line in stdout.lines().rev() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(env) = serde_json::from_str::<Envelope>(trimmed) {
                return Ok(env);
            }
        }
        Err(ScoreProcessError::ParseEnvelope {
            source: serde_json::from_str::<Envelope>(stdout.trim()).unwrap_err(),
            stdout,
        })
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl ScoreProcess for RealScoreProcess {
    async fn create(&self, title: &str) -> Result<Envelope, ScoreProcessError> {
        self.run(&["issues", "create", title]).await
    }

    async fn fill_section_apply(
        &self,
        slug: &str,
        section: &str,
        body: &str,
    ) -> Result<Envelope, ScoreProcessError> {
        self.run(&[
            "issues",
            "fill-section",
            "--apply",
            "--slug",
            slug,
            "--section",
            section,
            "--body",
            body,
        ])
        .await
    }

    async fn validate(&self, slug: &str) -> Result<Envelope, ScoreProcessError> {
        self.run(&["issues", "validate", slug]).await
    }

    async fn review_apply(&self, slug: &str, body: &str) -> Result<Envelope, ScoreProcessError> {
        self.run(&[
            "issues", "review", "--apply", "--slug", slug, "--body", body,
        ])
        .await
    }

    async fn merge(&self, slug: &str) -> Result<Envelope, ScoreProcessError> {
        self.run(&["issues", "merge", slug]).await
    }
}

/// Mock impl — feed it canned envelopes and assert on recorded calls.
#[derive(Debug, Clone, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub enum ScoreCall {
    Create {
        title: String,
    },
    FillSectionApply {
        slug: String,
        section: String,
        body: String,
    },
    ReviewApply {
        slug: String,
        body: String,
    },
    Validate {
        slug: String,
    },
    Merge {
        slug: String,
    },
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl ScoreCall {
    /// Stable verb token matching the CLI subcommand path. Lifecycle harness
    /// uses this to wait for "the next time MockScoreProcess sees `validate`".
    /// Tokens: `"create"`, `"fill_section"`, `"review_apply"`, `"validate"`,
    /// `"merge"`.
    pub fn verb(&self) -> &'static str {
        match self {
            ScoreCall::Create { .. } => "create",
            ScoreCall::FillSectionApply { .. } => "fill_section",
            ScoreCall::ReviewApply { .. } => "review_apply",
            ScoreCall::Validate { .. } => "validate",
            ScoreCall::Merge { .. } => "merge",
        }
    }
}

#[derive(Default)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub struct MockScoreProcess {
    create_responses: Mutex<Vec<Result<Envelope, ScoreProcessError>>>,
    fill_responses: Mutex<Vec<Result<Envelope, ScoreProcessError>>>,
    review_responses: Mutex<Vec<Result<Envelope, ScoreProcessError>>>,
    validate_responses: Mutex<Vec<Result<Envelope, ScoreProcessError>>>,
    merge_responses: Mutex<Vec<Result<Envelope, ScoreProcessError>>>,
    calls: Mutex<Vec<ScoreCall>>,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl MockScoreProcess {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue_create(&self, env: Envelope) -> &Self {
        self.create_responses.lock().unwrap().push(Ok(env));
        self
    }

    pub fn enqueue_create_err(&self, err: ScoreProcessError) -> &Self {
        self.create_responses.lock().unwrap().push(Err(err));
        self
    }

    pub fn enqueue_fill_section(&self, env: Envelope) -> &Self {
        self.fill_responses.lock().unwrap().push(Ok(env));
        self
    }

    pub fn enqueue_review(&self, env: Envelope) -> &Self {
        self.review_responses.lock().unwrap().push(Ok(env));
        self
    }

    pub fn enqueue_validate(&self, env: Envelope) -> &Self {
        self.validate_responses.lock().unwrap().push(Ok(env));
        self
    }

    pub fn enqueue_merge(&self, env: Envelope) -> &Self {
        self.merge_responses.lock().unwrap().push(Ok(env));
        self
    }

    pub fn calls(&self) -> Vec<ScoreCall> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl ScoreProcess for MockScoreProcess {
    async fn create(&self, title: &str) -> Result<Envelope, ScoreProcessError> {
        self.calls.lock().unwrap().push(ScoreCall::Create {
            title: title.to_string(),
        });
        let mut q = self.create_responses.lock().unwrap();
        if q.is_empty() {
            return Err(ScoreProcessError::MockExhausted("create".into()));
        }
        q.remove(0)
    }

    async fn fill_section_apply(
        &self,
        slug: &str,
        section: &str,
        body: &str,
    ) -> Result<Envelope, ScoreProcessError> {
        self.calls
            .lock()
            .unwrap()
            .push(ScoreCall::FillSectionApply {
                slug: slug.to_string(),
                section: section.to_string(),
                body: body.to_string(),
            });
        let mut q = self.fill_responses.lock().unwrap();
        if q.is_empty() {
            return Err(ScoreProcessError::MockExhausted(
                "fill_section_apply".into(),
            ));
        }
        q.remove(0)
    }

    async fn validate(&self, slug: &str) -> Result<Envelope, ScoreProcessError> {
        self.calls.lock().unwrap().push(ScoreCall::Validate {
            slug: slug.to_string(),
        });
        let mut q = self.validate_responses.lock().unwrap();
        if q.is_empty() {
            return Err(ScoreProcessError::MockExhausted("validate".into()));
        }
        q.remove(0)
    }

    async fn review_apply(&self, slug: &str, body: &str) -> Result<Envelope, ScoreProcessError> {
        self.calls.lock().unwrap().push(ScoreCall::ReviewApply {
            slug: slug.to_string(),
            body: body.to_string(),
        });
        let mut q = self.review_responses.lock().unwrap();
        if q.is_empty() {
            return Err(ScoreProcessError::MockExhausted("review_apply".into()));
        }
        q.remove(0)
    }

    async fn merge(&self, slug: &str) -> Result<Envelope, ScoreProcessError> {
        self.calls.lock().unwrap().push(ScoreCall::Merge {
            slug: slug.to_string(),
        });
        let mut q = self.merge_responses.lock().unwrap();
        if q.is_empty() {
            return Err(ScoreProcessError::MockExhausted("merge".into()));
        }
        q.remove(0)
    }
}

// ── IssueBackend impls ──────────────────────────────────────────────

/// Local SDD-file backend. Wraps an inner `Arc<dyn ScoreProcess>` for lifecycle
/// operations while using the local issue store for read/list/close.
///
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#changes
pub struct LocalIssueBackend {
    inner: Arc<dyn ScoreProcess>,
    issues_dir: PathBuf,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl LocalIssueBackend {
    pub fn new(inner: Arc<dyn ScoreProcess>) -> Self {
        let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::with_project_root(inner, project_root)
    }

    pub fn with_project_root(inner: Arc<dyn ScoreProcess>, project_root: impl AsRef<Path>) -> Self {
        Self::with_issues_dir(
            inner,
            crate::shared::workspace::issues_path(project_root.as_ref()),
        )
    }

    pub fn with_issues_dir(inner: Arc<dyn ScoreProcess>, issues_dir: impl Into<PathBuf>) -> Self {
        Self {
            inner,
            issues_dir: issues_dir.into(),
        }
    }

    fn local_store(&self) -> LocalBackend {
        LocalBackend::at(self.issues_dir.clone())
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl IssueBackend for LocalIssueBackend {
    fn backend_kind(&self) -> BackendKind {
        BackendKind::Local
    }

    async fn create(&self, title: &str) -> Result<IssueId, BackendError> {
        let env = self
            .inner
            .create(title)
            .await
            .map_err(|e| BackendError::Internal(format!("score process: {e}")))?;
        let slug = envelope_slug(&env).ok_or_else(|| {
            BackendError::Internal("score create returned envelope with no slug".into())
        })?;
        Ok(IssueId::new(slug))
    }

    async fn list(&self, filter: &ListFilter) -> Result<Vec<IssueRef>, BackendError> {
        let store = self.local_store();
        let issues = StoredIssueBackend::list(&store, &StoredIssueFilter::default())
            .await
            .map_err(local_store_error)?;

        Ok(issues
            .into_iter()
            .filter(|issue| runtime_filter_matches(issue, filter))
            .map(issue_ref_from_stored)
            .collect())
    }

    async fn read(&self, id: &IssueId) -> Result<IssueBody, BackendError> {
        let store = self.local_store();
        let issue = StoredIssueBackend::get(&store, id.as_str())
            .await
            .map_err(local_store_error)?
            .ok_or_else(|| BackendError::NotFound(id.clone()))?;
        issue_body_from_stored(&issue)
    }

    async fn update(&self, id: &IssueId, section: &str, body: &str) -> Result<(), BackendError> {
        self.inner
            .fill_section_apply(id.as_str(), section, body)
            .await
            .map(|_| ())
            .map_err(|e| BackendError::Internal(format!("fill_section_apply: {e}")))
    }

    async fn close(&self, id: &IssueId, message: Option<&str>) -> Result<(), BackendError> {
        let store = self.local_store();
        if StoredIssueBackend::get(&store, id.as_str())
            .await
            .map_err(local_store_error)?
            .is_none()
        {
            return Err(BackendError::NotFound(id.clone()));
        }

        StoredIssueBackend::close(&store, id.as_str(), message)
            .await
            .map_err(local_store_error)
    }
}

fn runtime_filter_matches(issue: &StoredIssue, filter: &ListFilter) -> bool {
    if stored_state_to_runtime(issue.state) != filter.state {
        return false;
    }

    filter
        .labels
        .iter()
        .all(|needle| issue.labels.iter().any(|label| label == needle))
}

fn stored_state_to_runtime(state: StoredIssueState) -> RuntimeIssueState {
    match state {
        StoredIssueState::Closed => RuntimeIssueState::Closed,
        StoredIssueState::Open | StoredIssueState::Draft => RuntimeIssueState::Open,
    }
}

fn issue_ref_from_stored(issue: StoredIssue) -> IssueRef {
    IssueRef {
        id: IssueId::new(issue.slug),
        title: issue.title,
        state: stored_state_to_runtime(issue.state),
        labels: issue.labels,
    }
}

fn issue_body_from_stored(issue: &StoredIssue) -> Result<IssueBody, BackendError> {
    Ok(IssueBody {
        id: IssueId::new(issue.slug.clone()),
        title: issue.title.clone(),
        body_md: issue.body.clone(),
        frontmatter: frontmatter_from_stored(issue)?,
    })
}

fn frontmatter_from_stored(
    issue: &StoredIssue,
) -> Result<BTreeMap<String, serde_json::Value>, BackendError> {
    let value = serde_json::to_value(issue)
        .map_err(|e| BackendError::Internal(format!("local issue frontmatter: {e}")))?;
    match value {
        serde_json::Value::Object(map) => Ok(map.into_iter().collect()),
        _ => Err(BackendError::Internal(
            "local issue did not serialize to an object".into(),
        )),
    }
}

fn local_store_error(error: anyhow::Error) -> BackendError {
    BackendError::Internal(format!("local issue store: {error}"))
}

/// Mock backend for tests — records every call, returns canned results
/// from queues. Default behavior: `create` pops from a queue of
/// canned ids; if the queue is empty, returns `BackendError::Internal`.
#[derive(Default)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub struct MockIssueBackend {
    kind: BackendKind,
    create_responses: Mutex<Vec<Result<IssueId, BackendError>>>,
    list_responses: Mutex<Vec<Result<Vec<IssueRef>, BackendError>>>,
    read_responses: Mutex<Vec<Result<IssueBody, BackendError>>>,
    calls: Mutex<Vec<MockBackendCall>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub enum MockBackendCall {
    Create {
        title: String,
    },
    List {
        filter: ListFilter,
    },
    Read {
        id: IssueId,
    },
    Update {
        id: IssueId,
        section: String,
        body: String,
    },
    Close {
        id: IssueId,
        message: Option<String>,
    },
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl MockIssueBackend {
    pub fn new(kind: BackendKind) -> Self {
        Self {
            kind,
            ..Self::default()
        }
    }

    pub fn enqueue_create(&self, id: IssueId) -> &Self {
        self.create_responses.lock().unwrap().push(Ok(id));
        self
    }

    pub fn enqueue_create_err(&self, err: BackendError) -> &Self {
        self.create_responses.lock().unwrap().push(Err(err));
        self
    }

    pub fn enqueue_list(&self, refs: Vec<IssueRef>) -> &Self {
        self.list_responses.lock().unwrap().push(Ok(refs));
        self
    }

    pub fn enqueue_read(&self, body: IssueBody) -> &Self {
        self.read_responses.lock().unwrap().push(Ok(body));
        self
    }

    pub fn calls(&self) -> Vec<MockBackendCall> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl IssueBackend for MockIssueBackend {
    fn backend_kind(&self) -> BackendKind {
        self.kind
    }

    async fn create(&self, title: &str) -> Result<IssueId, BackendError> {
        self.calls.lock().unwrap().push(MockBackendCall::Create {
            title: title.to_string(),
        });
        let mut q = self.create_responses.lock().unwrap();
        if q.is_empty() {
            return Err(BackendError::Internal(
                "MockIssueBackend: no canned create".into(),
            ));
        }
        q.remove(0)
    }

    async fn list(&self, filter: &ListFilter) -> Result<Vec<IssueRef>, BackendError> {
        self.calls.lock().unwrap().push(MockBackendCall::List {
            filter: filter.clone(),
        });
        let mut q = self.list_responses.lock().unwrap();
        if q.is_empty() {
            return Ok(vec![]);
        }
        q.remove(0)
    }

    async fn read(&self, id: &IssueId) -> Result<IssueBody, BackendError> {
        self.calls
            .lock()
            .unwrap()
            .push(MockBackendCall::Read { id: id.clone() });
        let mut q = self.read_responses.lock().unwrap();
        if q.is_empty() {
            return Err(BackendError::NotFound(id.clone()));
        }
        q.remove(0)
    }

    async fn update(&self, id: &IssueId, section: &str, body: &str) -> Result<(), BackendError> {
        self.calls.lock().unwrap().push(MockBackendCall::Update {
            id: id.clone(),
            section: section.to_string(),
            body: body.to_string(),
        });
        Ok(())
    }

    async fn close(&self, id: &IssueId, message: Option<&str>) -> Result<(), BackendError> {
        self.calls.lock().unwrap().push(MockBackendCall::Close {
            id: id.clone(),
            message: message.map(|s| s.to_string()),
        });
        Ok(())
    }
}

/// Sanity helper used by router/session impls: pull the slug out of a
/// `Dispatch`/`Done`/`Error` envelope. Returns None for `Batch`.
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub fn envelope_slug(env: &Envelope) -> Option<&str> {
    match env {
        Envelope::Dispatch { slug, .. }
        | Envelope::Done { slug, .. }
        | Envelope::Error { slug, .. } => Some(slug.as_str()),
        Envelope::Batch { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn done(slug: &str) -> Envelope {
        Envelope::Done {
            slug: slug.into(),
            message: None,
        }
    }

    #[tokio::test]
    async fn mock_records_create_call_and_returns_canned() {
        let mock = MockScoreProcess::new();
        mock.enqueue_create(done("abc"));
        let env = mock.create("hello world").await.unwrap();
        assert_eq!(envelope_slug(&env), Some("abc"));
        assert_eq!(
            mock.calls(),
            vec![ScoreCall::Create {
                title: "hello world".into()
            }]
        );
    }

    #[tokio::test]
    async fn mock_records_fill_section_call() {
        let mock = MockScoreProcess::new();
        mock.enqueue_fill_section(done("abc"));
        mock.fill_section_apply("abc", "requirements", "## body")
            .await
            .unwrap();
        assert_eq!(
            mock.calls(),
            vec![ScoreCall::FillSectionApply {
                slug: "abc".into(),
                section: "requirements".into(),
                body: "## body".into(),
            }]
        );
    }

    #[tokio::test]
    async fn mock_exhausted_returns_err() {
        let mock = MockScoreProcess::new();
        let err = mock.create("x").await.unwrap_err();
        assert!(matches!(err, ScoreProcessError::MockExhausted(_)));
    }

    fn stored_issue(slug: &str, state: StoredIssueState, labels: Vec<&str>) -> StoredIssue {
        StoredIssue {
            issue_type: crate::issues::IssueType::Enhancement,
            title: format!("test {slug}"),
            state,
            id: None,
            github_id: None,
            gitlab_id: None,
            url: None,
            author: Some("tester".into()),
            labels: labels.into_iter().map(str::to_string).collect(),
            created_at: None,
            updated_at: None,
            slug: slug.to_string(),
            body: "## Problem\n\nBody content.".into(),
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

    async fn write_stored_issue(issues_dir: &std::path::Path, issue: &StoredIssue) {
        let store = LocalBackend::at(issues_dir.to_path_buf());
        StoredIssueBackend::write(&store, issue).await.unwrap();
    }

    fn local_issue_backend(issues_dir: &std::path::Path) -> LocalIssueBackend {
        LocalIssueBackend::with_issues_dir(
            Arc::new(MockScoreProcess::new()),
            issues_dir.to_path_buf(),
        )
    }

    #[tokio::test]
    async fn local_issue_backend_lists_open_and_draft_by_label() {
        let tmp = tempfile::TempDir::new().unwrap();
        write_stored_issue(
            tmp.path(),
            &stored_issue(
                "open-jet",
                StoredIssueState::Open,
                vec!["project:jet", "priority:p1"],
            ),
        )
        .await;
        write_stored_issue(
            tmp.path(),
            &stored_issue(
                "draft-jet",
                StoredIssueState::Draft,
                vec!["project:jet", "priority:p1"],
            ),
        )
        .await;
        write_stored_issue(
            tmp.path(),
            &stored_issue(
                "closed-jet",
                StoredIssueState::Closed,
                vec!["project:jet", "priority:p1"],
            ),
        )
        .await;
        write_stored_issue(
            tmp.path(),
            &stored_issue("open-other", StoredIssueState::Open, vec!["project:jet"]),
        )
        .await;

        let backend = local_issue_backend(tmp.path());
        let refs = backend
            .list(&ListFilter {
                state: RuntimeIssueState::Open,
                labels: vec!["project:jet".into(), "priority:p1".into()],
            })
            .await
            .unwrap();
        let ids: Vec<_> = refs.into_iter().map(|r| r.id.0).collect();

        assert_eq!(ids, vec!["draft-jet", "open-jet"]);
    }

    #[tokio::test]
    async fn local_issue_backend_reads_body_and_frontmatter() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut issue = stored_issue(
            "enhancement-read",
            StoredIssueState::Open,
            vec!["project:jet"],
        );
        issue.phase = Some("reviewed".into());
        issue.review_count = Some(1);
        write_stored_issue(tmp.path(), &issue).await;

        let backend = local_issue_backend(tmp.path());
        let body = backend
            .read(&IssueId::new("enhancement-read"))
            .await
            .unwrap();

        assert_eq!(body.id.as_str(), "enhancement-read");
        assert!(body.body_md.contains("Body content"));
        assert_eq!(
            body.frontmatter.get("phase").and_then(|v| v.as_str()),
            Some("reviewed")
        );
        assert_eq!(
            body.frontmatter
                .get("review_count")
                .and_then(|v| v.as_u64()),
            Some(1)
        );
    }

    #[tokio::test]
    async fn local_issue_backend_read_missing_returns_not_found() {
        let tmp = tempfile::TempDir::new().unwrap();
        let backend = local_issue_backend(tmp.path());

        let err = backend.read(&IssueId::new("missing")).await.unwrap_err();

        assert!(matches!(err, BackendError::NotFound(id) if id.as_str() == "missing"));
    }

    #[tokio::test]
    async fn local_issue_backend_closes_existing_issue() {
        let tmp = tempfile::TempDir::new().unwrap();
        write_stored_issue(
            tmp.path(),
            &stored_issue("enhancement-close", StoredIssueState::Open, vec![]),
        )
        .await;

        let backend = local_issue_backend(tmp.path());
        backend
            .close(&IssueId::new("enhancement-close"), Some("done"))
            .await
            .unwrap();

        let open = backend.list(&ListFilter::default()).await.unwrap();
        assert!(open.is_empty());

        let closed = backend
            .list(&ListFilter {
                state: RuntimeIssueState::Closed,
                labels: vec![],
            })
            .await
            .unwrap();
        assert_eq!(closed.len(), 1);
        assert_eq!(closed[0].id.as_str(), "enhancement-close");
        assert_eq!(closed[0].state, RuntimeIssueState::Closed);
    }
}

// CODEGEN-END
