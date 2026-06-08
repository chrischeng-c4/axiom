---
id: projects-sdd-tests-gitlab-backend-tests-rs
fill_sections: [overview, tests, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue/backend validation TDs verify AW Core client boundary behavior."
---

# GitLab Backend Integration Tests

## Overview
<!-- type: overview lang: markdown -->

Codegenerated async integration tests for `GitLabIssueBackend` subprocess
execution, auth handling, list-state and label mapping, read-body mapping, and
unsupported update behavior. The tests use a shell-script mock `glab` binary
so no network access is required.

## Tests
<!-- type: tests lang: yaml -->

```yaml
preamble: |
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
imports: []
tests:
  - name: create_happy_path
    attributes:
      - "#[tokio::test]"
    async: true
    body: |
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
  - name: auth_missing_returns_error
    attributes:
      - "#[tokio::test]"
    async: true
    body: |
      let _g = env_lock().lock().unwrap_or_else(|e| e.into_inner());
      std::env::remove_var("GITLAB_TOKEN");
      
      let result = GitLabIssueBackend::from_env(None);
      match result {
          Err(BackendError::Auth(msg)) => assert!(msg.contains("GITLAB_TOKEN")),
          other => panic!("expected Auth error; got {other:?}"),
      }
  - name: list_open_issues_maps_state_and_labels
    attributes:
      - "#[tokio::test]"
    async: true
    body: |
      let _g = env_lock().lock().unwrap_or_else(|e| e.into_inner());
      std::env::set_var("GITLAB_TOKEN", "test-token");
      
      let tmp = TempDir::new().unwrap();
      let mock_value = serde_json::json!([
          { "iid": 1, "title": "first",  "state": "opened", "labels": ["bug", "p0"] },
          { "iid": 2, "title": "second", "state": "closed", "labels": [] },
      ]);
      let mock = write_mock_glab(&tmp, &mock_value.to_string());
      let backend = GitLabIssueBackend::from_env(None)
          .unwrap()
          .with_binary(mock.to_string_lossy());
      
      let refs = backend.list(&ListFilter::default()).await.expect("list");
      assert_eq!(refs.len(), 2);
      assert_eq!(refs[0].id, IssueId::new("1"));
      assert_eq!(refs[0].state, IssueState::Open);
      assert_eq!(refs[0].labels, vec!["bug", "p0"]);
      assert_eq!(refs[1].state, IssueState::Closed);
      
      std::env::remove_var("GITLAB_TOKEN");
  - name: read_by_id_returns_body
    attributes:
      - "#[tokio::test]"
    async: true
    indent_body: false
    body: |
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
  - name: update_returns_unsupported_in_slice_1
    attributes:
      - "#[tokio::test]"
    async: true
    body: |
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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/gitlab_backend_tests.rs
    action: modify
    section: tests
    impl_mode: codegen
    generator: rust.tests
    description: |
      Emit the GitLab backend async tests from the Rust tests template with
      top-level helper preamble and tokio test attributes.
```
