//! E2E tests for IssueBackend routing — proves `Action::SubmitChat`
//! flows through `Session::decide` → `MainthreadDecision::NewIssue` →
//! `active_backend.create`, NOT directly through ScoreProcess.
//!
//! @spec projects/agentic-workflow/tech-design/core/runtime/issue_backend.md#scenarios
//! Scenario `mainthread_routes_through_active_backend`.
//!
//! Slice 1: only the routing wiring is tested. Real github/gitlab/jira
//! impls land in follow-up commits.

mod support;

use crossterm::event::KeyCode;
use sdd::runtime::{
    BackendKind, IssueId, MockBackendCall, MockIssueBackend, Session, StaticRouter,
};
use std::sync::Arc;
use std::time::Duration;
use support::harness::CueHarness;
use support::mock_llm::{final_chunk, MockLlmProvider};

const QUICK: Duration = Duration::from_millis(2000);

#[tokio::test]
async fn newissue_routes_to_active_backend() {
    // GitHub-flavored MockIssueBackend; cue should call IT, not the
    // local ScoreProcess, when active_backend is set to it.
    let mock_backend = Arc::new(MockIssueBackend::new(BackendKind::GitHub));
    mock_backend.enqueue_create(IssueId::new("42"));

    // Build a minimal Session manually — the harness builder hardwires
    // a LocalIssueBackend. We need to inject the mock backend, so
    // construct via SessionBuilder directly.
    let llm = Arc::new(MockLlmProvider::new("mock"));
    // Mainthread JSON: NewIssue intent.
    llm.enqueue_stream(vec![final_chunk(
        r#"{"action":"new_issue","title":"dashboard widget"}"#,
    )]);

    let score_process = Arc::new(sdd::runtime::MockScoreProcess::new());
    // The local score binary must NOT receive a create call when the
    // active backend is GitHub-flavored. Empty queue → MockExhausted
    // if anything tries to call score_process.create — but the test
    // asserts zero score_calls instead, which is more direct.

    let session = Session::builder()
        .provider(llm.clone())
        .aw_process(score_process.clone())
        .issue_backend(mock_backend.clone())
        .router(Arc::new(StaticRouter::defaults()))
        .build()
        .expect("build session");

    let mut h = CueHarness::from_session(session);
    h.press(KeyCode::Char('n')).await;
    h.type_str("add a dashboard widget").await;
    h.press(KeyCode::Enter).await;
    h.run_until_idle().await;

    // Assertion 1: MockIssueBackend got exactly one Create call with
    // the title the mainthread classified.
    let backend_calls = mock_backend.calls();
    assert_eq!(
        backend_calls.len(),
        1,
        "expected exactly one IssueBackend::create call; got: {backend_calls:?}"
    );
    match &backend_calls[0] {
        MockBackendCall::Create { title } => {
            assert_eq!(title, "dashboard widget");
        }
        other => panic!("expected Create call; got {other:?}"),
    }

    // Assertion 2: ScoreProcess::create was NOT called directly. Remote
    // backend → no SDD CRRR lifecycle in slice 1 per R9.
    assert!(
        score_process.calls().is_empty(),
        "score binary must not receive a create call when active backend is remote; got: {:?}",
        score_process.calls()
    );
}

#[tokio::test]
async fn local_backend_e2e_no_regression() {
    // Default builder uses LocalIssueBackend wrapping MockScoreProcess.
    // This test exercises the same flow as `e2e_lifecycle.rs::slice1_*`
    // but via the new IssueBackend routing — proves the local path
    // still ends-to-end correctly through the trait.
    let mut h = CueHarness::builder()
        .with_score_create(sdd::runtime::Envelope::Dispatch {
            slug: "abc".into(),
            agent: Some("score-issue-author".into()),
            invoke: Some(sdd::runtime::Invoke {
                command: "aw wi fill-section".into(),
                args: serde_json::json!({"slug": "abc", "sections": ["requirements"]}),
            }),
        })
        .with_score_fill(sdd::runtime::Envelope::Done {
            slug: "abc".into(),
            message: Some("ok".into()),
        })
        .with_mainthread_new_issue("local issue")
        .with_llm_chunks(vec![final_chunk("requirements body")])
        .build();

    h.press(KeyCode::Char('n')).await;
    h.type_str("local issue").await;
    h.press(KeyCode::Enter).await;

    h.expect_next_score_call("create", QUICK).await.unwrap();
    h.expect_next_score_call("fill_section", QUICK)
        .await
        .unwrap();
    h.run_until_idle().await;

    // Local lifecycle reached terminal Merged state.
    assert_eq!(
        h.app().lifecycle_state,
        cue::tui::app::LifecycleState::Merged,
        "local backend should still reach Merged via the new trait dispatch"
    );
}
