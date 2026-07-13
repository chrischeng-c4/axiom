// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/validate/tests/wi_close_remote_test.md#source
// CODEGEN-BEGIN
//! Real-CLI coverage for remote numeric `aw wi close` rehydration (#1551).
//!
//! The subprocess backend is isolated from GitHub: a temp-HOME `gh` adapter
//! forwards every invocation to an in-process HTTP fixture server. Tests can
//! therefore assert remote reads, comments, close count, and explicit repo
//! routing without mutating any real tracker issue.

use agentic_workflow::issues::{remote_read_cache_backend, IssueBackend, IssueState, LocalBackend};
use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

#[derive(Clone, Default)]
struct GhFixtureState {
    calls: Arc<Mutex<Vec<String>>>,
    closed: Arc<AtomicBool>,
}

async fn gh_fixture(State(state): State<GhFixtureState>, body: String) -> Response<Body> {
    state.calls.lock().unwrap().push(body.clone());

    if body.contains(" view 404 ") {
        return (StatusCode::NOT_FOUND, "fixture issue not found").into_response();
    }

    if body.contains(" view 42 ") {
        let issue_state = if state.closed.load(Ordering::SeqCst) {
            "CLOSED"
        } else {
            "OPEN"
        };
        let payload = serde_json::json!({
            "number": 42,
            "title": "remote fixture",
            "state": issue_state,
            "labels": [{"name": "type:bug"}],
            "author": {"login": "fixture"},
            "createdAt": "2026-07-13T00:00:00Z",
            "updatedAt": "2026-07-13T00:00:00Z",
            "url": "https://example.invalid/fixture/issues/42",
            "body": "## Scope\n\nremote-only fixture",
        });
        return (StatusCode::OK, payload.to_string()).into_response();
    }

    if body.contains(" state=closed") {
        state.closed.store(true, Ordering::SeqCst);
        return (StatusCode::OK, "{}").into_response();
    }

    if body.contains("/comments") {
        return (StatusCode::OK, "{}").into_response();
    }

    (
        StatusCode::BAD_REQUEST,
        format!("unexpected fixture gh invocation: {body}"),
    )
        .into_response()
}

async fn start_gh_fixture() -> (String, GhFixtureState, tokio::task::JoinHandle<()>) {
    let state = GhFixtureState::default();
    let app = Router::new()
        .route("/gh", post(gh_fixture))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (url, state, handle)
}

fn write_project(root: &Path) {
    fs::write(
        root.join("aw.toml"),
        concat!(
            "[agentic_workflow.issue_platform]\n",
            "type = \"github\"\n",
            "repo = \"fixture/configured\"\n",
        ),
    )
    .unwrap();
}

fn write_gh_adapter(root: &Path) -> PathBuf {
    let gh = root
        .join("home")
        .join(".rustup/toolchains/stable-aarch64-apple-darwin/bin/gh");
    fs::create_dir_all(gh.parent().unwrap()).unwrap();
    fs::write(
        &gh,
        concat!(
            "#!/bin/sh\n",
            "exec /usr/bin/curl --silent --show-error --fail -X POST ",
            "--data \"$*\" \"$AW_GH_FIXTURE_URL/gh\"\n",
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(&gh).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&gh, permissions).unwrap();
    gh
}

fn run_aw(root: &Path, fixture_url: &str, args: &[&str]) -> Output {
    let gh = write_gh_adapter(root);
    let home = gh.ancestors().nth(5).expect("temp HOME from gh path");
    Command::new(env!("CARGO_BIN_EXE_aw"))
        .args(args)
        .current_dir(root)
        .env("HOME", home)
        .env("GH_TOKEN", "fixture-token")
        .env("AW_GH_FIXTURE_URL", fixture_url)
        .env("AW_DISABLE_CAP", "1")
        .output()
        .expect("run repo-built aw")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wi_close_remote_numeric_rehydrates_reason_and_closes_once() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    let (fixture_url, state, server) = start_gh_fixture().await;
    let repo = "fixture/explicit-1551";
    let cache = remote_read_cache_backend("github", Some(repo), None);
    let _ = fs::remove_dir_all(cache.issues_dir());

    let show = run_aw(
        root.path(),
        &fixture_url,
        &["wi", "show", "42", "--repo", repo],
    );
    assert!(
        show.status.success(),
        "remote show must resolve before close:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&show.stdout),
        String::from_utf8_lossy(&show.stderr),
    );
    let _ = fs::remove_dir_all(cache.issues_dir());
    assert!(
        !cache.issues_dir().exists(),
        "close must start without a /tmp remote-read cache"
    );

    for attempt in 1..=2 {
        let close = run_aw(
            root.path(),
            &fixture_url,
            &[
                "wi",
                "close",
                "42",
                "--push",
                "--repo",
                repo,
                "--reason",
                "fixture reason",
            ],
        );
        assert!(
            close.status.success(),
            "remote close attempt {attempt} must succeed:\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&close.stdout),
            String::from_utf8_lossy(&close.stderr),
        );
        assert_eq!(String::from_utf8_lossy(&close.stdout).trim(), "Closed 42");
    }

    let calls = state.calls.lock().unwrap().clone();
    assert_eq!(
        calls
            .iter()
            .filter(|call| call.contains(" view 42 "))
            .count(),
        3,
        "show plus two close preflights must use the remote backend: {calls:#?}"
    );
    assert_eq!(
        calls
            .iter()
            .filter(
                |call| call.contains("api -X PATCH repos/fixture/explicit-1551/issues/42")
                    && call.contains("state=closed")
            )
            .count(),
        1,
        "the remote issue must be closed exactly once: {calls:#?}"
    );
    assert_eq!(
        calls
            .iter()
            .filter(|call| call
                .contains("api -X POST repos/fixture/explicit-1551/issues/42/comments")
                && call.contains("body=fixture reason"))
            .count(),
        1,
        "the optional reason must be posted exactly once: {calls:#?}"
    );
    assert!(
        calls
            .iter()
            .filter(|call| call.contains(" view 42 "))
            .all(|call| call.contains("--repo fixture/explicit-1551")),
        "--repo must select the remote for every read: {calls:#?}"
    );

    let cached = cache.get("42").await.unwrap().expect("rehydrated cache");
    assert_eq!(cached.state, IssueState::Closed);
    let _ = fs::remove_dir_all(cache.issues_dir());
    server.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wi_close_missing_remote_reports_backend_repo_and_recovery_command() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    let (fixture_url, state, server) = start_gh_fixture().await;
    let repo = "fixture/missing-1551";

    let close = run_aw(
        root.path(),
        &fixture_url,
        &["wi", "close", "404", "--push", "--repo", repo, "--json"],
    );
    assert_eq!(close.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&close.stderr);
    assert!(stderr.contains("\"code\":\"NOT_FOUND\""), "{stderr}");
    assert!(stderr.contains("github backend"), "{stderr}");
    assert!(
        stderr.contains("repository 'fixture/missing-1551'"),
        "{stderr}"
    );
    assert!(
        stderr.contains("aw wi show 404 --repo fixture/missing-1551"),
        "{stderr}"
    );

    let calls = state.calls.lock().unwrap().clone();
    assert_eq!(calls.len(), 1, "missing lookup must not mutate: {calls:#?}");
    assert!(calls[0].contains("--repo fixture/missing-1551"));
    assert!(calls[0].contains(" view 404 "));
    server.abort();
}

#[test]
fn wi_close_local_issue_behavior_is_preserved() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    let local = LocalBackend::from_project_root(root.path());
    let open_dir = local.issues_dir().join("open");
    fs::create_dir_all(&open_dir).unwrap();
    fs::write(
        open_dir.join("local-only.md"),
        concat!(
            "---\n",
            "type: bug\n",
            "title: local close fixture\n",
            "state: open\n",
            "---\n",
            "local body\n",
        ),
    )
    .unwrap();

    let close = Command::new(env!("CARGO_BIN_EXE_aw"))
        .args(["wi", "close", "local-only"])
        .current_dir(root.path())
        .env("AW_DISABLE_CAP", "1")
        .output()
        .expect("run local close");
    assert!(
        close.status.success(),
        "local close must remain unchanged:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&close.stdout),
        String::from_utf8_lossy(&close.stderr),
    );
    assert!(local.issues_dir().join("closed/local-only.md").exists());
    assert!(!open_dir.join("local-only.md").exists());
    let _ = fs::remove_dir_all(local.issues_dir());
}
// CODEGEN-END
