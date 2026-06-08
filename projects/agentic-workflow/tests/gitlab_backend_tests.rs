// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/gitlab_backend_tests.md#tests
// CODEGEN-BEGIN

//! Integration tests for `GitLabIssueBackend`. Mirror of
//! `github_backend_tests.rs` adjusted for glab specifics.
//!
//! @spec projects/agentic-workflow/tech-design/core/runtime/gitlab_backend.md

use agentic_workflow::runtime::{
    BackendError, BackendKind, GitLabIssueBackend, IssueBackend, IssueId, IssueState, ListFilter,
};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn write_mock_glab(tempdir: &TempDir, stdout_text: &str) -> PathBuf {
    let path = tempdir.path().join("glab");
    let script = format!("#!/bin/sh\ncat <<'__EOF__'\n{stdout_text}\n__EOF__\n");
    fs::write(&path, script).expect("write mock glab");
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("chmod mock glab");
    path
}

fn write_recording_mock_glab(tempdir: &TempDir, stdout_text: &str) -> (PathBuf, PathBuf) {
    let path = tempdir.path().join("glab");
    let args_path = tempdir.path().join("glab.args");
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\ncat <<'__EOF__'\n{}\n__EOF__\n",
        args_path.display(),
        stdout_text
    );
    fs::write(&path, script).expect("write recording mock glab");
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("chmod recording mock glab");
    (path, args_path)
}

#[tokio::test]
async fn create_happy_path() {
    let _g = env_lock().lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("GITLAB_TOKEN", "test-token");

    let tmp = TempDir::new().unwrap();
    let mock_url = "https://gitlab.com/group/proj/-/issues/13";
    let mock = write_mock_glab(&tmp, mock_url);

    let backend = GitLabIssueBackend::from_env(None)
        .unwrap()
        .with_binary(mock.to_string_lossy());

    let id = backend.create("dashboard widget").await.expect("create");
    assert_eq!(id, IssueId::new("13"));
    assert_eq!(backend.backend_kind(), BackendKind::GitLab);

    std::env::remove_var("GITLAB_TOKEN");
}

#[tokio::test]
async fn auth_missing_returns_error() {
    let _g = env_lock().lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("GITLAB_TOKEN");

    let result = GitLabIssueBackend::from_env(None);
    match result {
        Err(BackendError::Auth(msg)) => assert!(msg.contains("GITLAB_TOKEN")),
        other => panic!("expected Auth error; got {other:?}"),
    }
}

#[tokio::test]
async fn list_open_issues_maps_state_and_labels() {
    let _g = env_lock().lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("GITLAB_TOKEN", "test-token");

    let tmp = TempDir::new().unwrap();
    let mock_value = serde_json::json!([
        { "iid": 1, "title": "first",  "state": "opened", "labels": ["bug", "p0"] },
        { "iid": 2, "title": "second", "state": "closed", "labels": [] },
    ]);
    let (mock, args_path) = write_recording_mock_glab(&tmp, &mock_value.to_string());
    let backend = GitLabIssueBackend::from_env(None)
        .unwrap()
        .with_binary(mock.to_string_lossy());

    let refs = backend.list(&ListFilter::default()).await.expect("list");
    let args = fs::read_to_string(args_path).expect("read glab args");
    assert!(args.contains("--output\njson\n"), "got args:\n{args}");
    assert!(!args.contains("--state\n"), "got args:\n{args}");
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0].id, IssueId::new("1"));
    assert_eq!(refs[0].state, IssueState::Open);
    assert_eq!(refs[0].labels, vec!["bug", "p0"]);
    assert_eq!(refs[1].state, IssueState::Closed);

    std::env::remove_var("GITLAB_TOKEN");
}

#[tokio::test]
async fn read_by_id_returns_body() {
    let _g = env_lock().lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("GITLAB_TOKEN", "test-token");

    let tmp = TempDir::new().unwrap();
    let mock_value = serde_json::json!({
        "iid": 7,
        "title": "the seven",
        "state": "opened",
        "labels": [],
        "description": "## Description\n\nFull body here.",
    });
    let mock = write_mock_glab(&tmp, &mock_value.to_string());
    let backend = GitLabIssueBackend::from_env(None)
        .unwrap()
        .with_binary(mock.to_string_lossy());

    let body = backend.read(&IssueId::new("7")).await.expect("read");
    assert_eq!(body.id, IssueId::new("7"));
    assert_eq!(body.title, "the seven");
    assert!(body.body_md.contains("Full body here"));
    assert!(body.frontmatter.is_empty());

    std::env::remove_var("GITLAB_TOKEN");
}

#[tokio::test]
async fn update_returns_unsupported_in_slice_1() {
    let _g = env_lock().lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("GITLAB_TOKEN", "test-token");

    let tmp = TempDir::new().unwrap();
    let mock = write_mock_glab(&tmp, "");
    let backend = GitLabIssueBackend::from_env(None)
        .unwrap()
        .with_binary(mock.to_string_lossy());

    let r = backend
        .update(&IssueId::new("1"), "requirements", "body")
        .await;
    assert!(matches!(r, Err(BackendError::Unsupported)));

    std::env::remove_var("GITLAB_TOKEN");
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// CODEGEN-BEGIN
// SPEC-REF: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// TODO: Existing source behavior is covered by this feature/domain semantic TD.

// CODEGEN-END
