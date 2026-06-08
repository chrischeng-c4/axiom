---
id: projects-sdd-tests-github-backend-tests-rs
fill_sections: [overview, tests, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue/backend validation TDs verify AW Core client boundary behavior."
---

# GitHub Backend Integration Tests

## Overview
<!-- type: overview lang: markdown -->

Codegenerated async integration tests for `GitHubIssueBackend` subprocess
execution, auth handling, list-state mapping, read-body mapping, and unsupported
update behavior. The test binary uses a shell-script mock `gh` executable, so
it exercises subprocess and JSON parsing paths without network access.

## Tests
<!-- type: tests lang: yaml -->

```yaml
preamble: |
  //! Integration tests for `GitHubIssueBackend`. Uses a shell-script
  //! "mock gh" written to a tempdir — exercises the subprocess + JSON
  //! parsing paths without network access.
  //!
  //! @spec projects/agentic-workflow/tech-design/core/runtime/github_backend.md
  //! Acceptance tests: `create_happy_path`, `auth_missing_returns_error`.
  
  use agentic_workflow::runtime::{
      BackendError, BackendKind, GitHubIssueBackend, IssueBackend, IssueId, IssueState, ListFilter,
  };
  use std::fs;
  use std::os::unix::fs::PermissionsExt;
  use std::path::PathBuf;
  use std::sync::{Mutex, OnceLock};
  use tempfile::TempDir;
  
  /// Shared lock — env vars are process-global, so cargo test's
  /// parallel execution causes flakes when tests mutate GITHUB_TOKEN.
  /// Serialize all GH backend tests via this guard.
  fn env_lock() -> &'static Mutex<()> {
      static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
      LOCK.get_or_init(|| Mutex::new(()))
  }
  
  /// Write a shell script to `<tempdir>/gh` that prints `stdout_text`
  /// and exits 0. Returns the absolute path to the script.
  fn write_mock_gh(tempdir: &TempDir, stdout_text: &str) -> PathBuf {
      let path = tempdir.path().join("gh");
      let script = format!("#!/bin/sh\ncat <<'__EOF__'\n{stdout_text}\n__EOF__\n");
      fs::write(&path, script).expect("write mock gh");
      let mut perms = fs::metadata(&path).unwrap().permissions();
      perms.set_mode(0o755);
      fs::set_permissions(&path, perms).expect("chmod mock gh");
      path
  }
  
  fn with_token<F: FnOnce()>(token: &str, f: F) {
      let prev = std::env::var("GITHUB_TOKEN").ok();
      std::env::set_var("GITHUB_TOKEN", token);
      f();
      match prev {
          Some(v) => std::env::set_var("GITHUB_TOKEN", v),
          None => std::env::remove_var("GITHUB_TOKEN"),
      }
  }
  
  fn without_token<F: FnOnce()>(f: F) {
      let prev = std::env::var("GITHUB_TOKEN").ok();
      std::env::remove_var("GITHUB_TOKEN");
      f();
      if let Some(v) = prev {
          std::env::set_var("GITHUB_TOKEN", v);
      }
  }
imports: []
tests:
  - name: create_happy_path
    attributes:
      - "#[tokio::test]"
    async: true
    body: |
      // Test deferred via std::sync::Mutex to keep env-var manipulation
      // serial — env vars are process-global.
      let _g = env_lock().lock().unwrap_or_else(|e| e.into_inner());
      
      with_token("test-token", || {});
      std::env::set_var("GITHUB_TOKEN", "test-token");
      
      let tmp = TempDir::new().unwrap();
      let mock_url = "https://github.com/owner/repo/issues/42";
      let mock_gh = write_mock_gh(&tmp, mock_url);
      
      let backend = GitHubIssueBackend::from_env(None)
          .expect("token set, construction OK")
          .with_binary(mock_gh.to_string_lossy());
      
      let id = backend.create("new widget").await.expect("create");
      assert_eq!(id, IssueId::new("42"));
      assert_eq!(backend.backend_kind(), BackendKind::GitHub);
      
      std::env::remove_var("GITHUB_TOKEN");
  - name: auth_missing_returns_error
    attributes:
      - "#[tokio::test]"
    async: true
    body: |
      let _g = env_lock().lock().unwrap_or_else(|e| e.into_inner());
      
      without_token(|| {
          // Construction itself fails — never spawns the subprocess.
          let result = GitHubIssueBackend::from_env(None);
          match result {
              Err(BackendError::Auth(msg)) => assert!(
                  msg.contains("GITHUB_TOKEN"),
                  "expected error message to mention GITHUB_TOKEN; got: {msg}"
              ),
              other => panic!("expected Auth error; got {other:?}"),
          }
      });
  - name: list_open_issues_maps_state
    attributes:
      - "#[tokio::test]"
    async: true
    body: |
      let _g = env_lock().lock().unwrap_or_else(|e| e.into_inner());
      std::env::set_var("GITHUB_TOKEN", "test-token");
      
      let tmp = TempDir::new().unwrap();
      let mock_value = serde_json::json!([
          { "number": 1, "title": "first",  "state": "OPEN",   "labels": [{ "name": "bug" }] },
          { "number": 2, "title": "second", "state": "CLOSED", "labels": [] },
      ]);
      let mock_json = mock_value.to_string();
      let mock_gh = write_mock_gh(&tmp, &mock_json);
      
      let backend = GitHubIssueBackend::from_env(None)
          .unwrap()
          .with_binary(mock_gh.to_string_lossy());
      
      let refs = backend.list(&ListFilter::default()).await.expect("list");
      assert_eq!(refs.len(), 2);
      assert_eq!(refs[0].state, IssueState::Open);
      assert_eq!(refs[0].labels, vec!["bug"]);
      assert_eq!(refs[1].state, IssueState::Closed);
      
      std::env::remove_var("GITHUB_TOKEN");
  - name: read_by_id_returns_body
    attributes:
      - "#[tokio::test]"
    async: true
    indent_body: false
    body: |
      let _g = env_lock().lock().unwrap_or_else(|e| e.into_inner());
      std::env::set_var("GITHUB_TOKEN", "test-token");
      
      let tmp = TempDir::new().unwrap();
      let mock_value = serde_json::json!({
          "number": 7,
          "title": "the seven",
          "state": "OPEN",
          "labels": [],
          "body": "## Description\n\nFull body here.",
      });
      let mock_json = mock_value.to_string();
      let mock_gh = write_mock_gh(&tmp, &mock_json);
      
      let backend = GitHubIssueBackend::from_env(None)
          .unwrap()
          .with_binary(mock_gh.to_string_lossy());
      
      let body = backend.read(&IssueId::new("7")).await.expect("read");
      assert_eq!(body.id, IssueId::new("7"));
      assert_eq!(body.title, "the seven");
      assert!(body.body_md.contains("Full body here"));
      assert!(body.frontmatter.is_empty(), "remote returns no frontmatter");
      
      std::env::remove_var("GITHUB_TOKEN");
  - name: update_returns_unsupported_in_slice_1
    attributes:
      - "#[tokio::test]"
    async: true
    body: |
      let _g = env_lock().lock().unwrap_or_else(|e| e.into_inner());
      std::env::set_var("GITHUB_TOKEN", "test-token");
      
      let tmp = TempDir::new().unwrap();
      let mock_gh = write_mock_gh(&tmp, "");
      let backend = GitHubIssueBackend::from_env(None)
          .unwrap()
          .with_binary(mock_gh.to_string_lossy());
      
      let r = backend
          .update(&IssueId::new("1"), "requirements", "body")
          .await;
      assert!(matches!(r, Err(BackendError::Unsupported)));
      
      std::env::remove_var("GITHUB_TOKEN");
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/github_backend_tests.rs
    action: modify
    section: tests
    impl_mode: codegen
    generator: rust.tests
    description: |
      Emit the GitHub backend async tests from the Rust tests template with
      top-level helper preamble and tokio test attributes.
```
