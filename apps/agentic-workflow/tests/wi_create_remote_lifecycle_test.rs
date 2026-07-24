// @spec apps/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R1 #R5 #R8
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:wi-create-remote-authoring-loop" tracker="#2289" reason="The fixture drives the compiled CLI through a remote tracker adapter and verifies the cross-process workspace handoff."

use agentic_workflow::issues::{remote_read_cache_backend, IssueBackend, LocalBackend};
use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

const REPO: &str = "fixture/wi-create-remote-2289";
const FILLED_BODY: &str = r#"## Problem

Remote tracker creation must remain inside the WI authoring loop.

## Capability Alignment

Capability: Work Item Planning
Capability Gap: Remote create did not expose the new tracker id to fill-section.
Progress Evidence: The compiled CLI completes create, fill, remote projection, and validate.

## Requirements

- R1: Preserve one executable authoring chain after remote creation.

## Scope

### In Scope
- Mirror the remote issue into workspace lifecycle state and project the accepted fill back.

### Out of Scope
- Changing tracker selection or issue type semantics.

## Acceptance Criteria

- AC1: Remote validate reads the authored body and emits a terminal done envelope.

## Reference Context

### Related Specs
| Spec | Relevance |
|------|-----------|
| issue-cli-envelope | Defines the WI authoring chain. |

### Spec Plan
| Spec ID | Action | Main Spec Ref |
|---------|--------|---------------|
| issue-cli-envelope | update | R1, R5, R8 |
"#;

#[derive(Clone, Default)]
struct GhFixtureState {
    calls: Arc<Mutex<Vec<String>>>,
    filled: Arc<AtomicBool>,
}

fn issue_json(body: &str) -> String {
    serde_json::json!({
        "number": 42,
        "title": "Remote create authoring fixture",
        "state": "OPEN",
        "labels": [
            {"name": "type:change"},
            {"name": "app:demo"},
            {"name": "priority:p1"},
            {"name": "phase:created"}
        ],
        "author": {"login": "fixture"},
        "createdAt": "2026-07-21T00:00:00Z",
        "updatedAt": "2026-07-21T00:00:00Z",
        "url": "https://example.invalid/fixture/issues/42",
        "body": body,
    })
    .to_string()
}

async fn gh_fixture(State(state): State<GhFixtureState>, body: String) -> Response<Body> {
    state.calls.lock().unwrap().push(body.clone());

    if body.starts_with("label create ") {
        return (StatusCode::OK, "{}").into_response();
    }

    if body.contains(&format!("api -X POST repos/{REPO}/issues ")) {
        return (StatusCode::OK, issue_json("initial remote body")).into_response();
    }

    if body.contains(" view 42 ") {
        let issue_body = if state.filled.load(Ordering::SeqCst) {
            FILLED_BODY
        } else {
            "initial remote body"
        };
        return (StatusCode::OK, issue_json(issue_body)).into_response();
    }

    if body.contains(&format!("api -X PATCH repos/{REPO}/issues/42")) {
        if body.contains("body=\n## Problem") {
            state.filled.store(true, Ordering::SeqCst);
        }
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
        format!(
            r#"
[agentic_workflow.workspace]
mode = "in_place"

[agentic_workflow.issue_platform]
type = "github"
repo = "{REPO}"

[[projects]]
name = "demo"
label = "app:demo"
path = "."
tech_design_path = "tech-design"

[[projects.workspaces]]
name = "demo"
paths = ["**"]
target = "rust"
"#,
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

fn successful_json(output: &Output, command: &str) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "{command} failed:\nstdout={stdout}\nstderr={stderr}"
    );
    serde_json::from_str(stdout.trim()).unwrap_or_else(|error| {
        panic!("{command} did not emit one JSON value: {error}\nstdout={stdout}\nstderr={stderr}")
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remote_create_stays_in_authoring_loop_until_remote_validate_is_done() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    let (fixture_url, state, server) = start_gh_fixture().await;
    let cache = remote_read_cache_backend("github", Some(REPO), None);
    let _ = fs::remove_dir_all(cache.issues_dir());

    let create = run_aw(
        root.path(),
        &fixture_url,
        &[
            "wi",
            "create",
            "--title",
            "Remote create authoring fixture",
            "--type",
            "change",
            "--project",
            "demo",
            "--priority",
            "p1",
        ],
    );
    let create_json = successful_json(&create, "aw wi create");
    assert_eq!(create_json["action"], "dispatch");
    assert_eq!(create_json["slug"], "42");
    assert_eq!(create_json["invoke"]["command"], "aw wi fill-section");

    let local = LocalBackend::from_project_root(root.path());
    let mirrored = local
        .get("42")
        .await
        .unwrap()
        .expect("remote issue must be mirrored for fill-section");
    assert_eq!(mirrored.github_id, Some(42));

    let payload = PathBuf::from(
        create_json["invoke"]["args"]["payload_path"]
            .as_str()
            .expect("create payload path"),
    );
    assert!(payload.exists(), "create must initialize the fill payload");
    fs::write(&payload, FILLED_BODY).unwrap();

    let fill = run_aw(
        root.path(),
        &fixture_url,
        &["wi", "fill-section", "--slug", "42", "--apply"],
    );
    let fill_json = successful_json(&fill, "aw wi fill-section --apply");
    assert_eq!(fill_json["action"], "dispatch");
    assert_eq!(fill_json["invoke"]["command"], "aw wi validate");
    assert!(
        state.filled.load(Ordering::SeqCst),
        "fill-section must project the authored body to the tracker: {:#?}",
        state.calls.lock().unwrap()
    );

    let validate = run_aw(root.path(), &fixture_url, &["wi", "validate", "42"]);
    let validate_json = successful_json(&validate, "aw wi validate 42");
    assert_eq!(validate_json["schema_version"], "aw.cli.v1");
    assert_eq!(validate_json["passed"], true);
    assert_eq!(validate_json["completion"]["workflow_complete"], true);
    assert_eq!(validate_json["next"]["kind"], "done");

    let calls = state.calls.lock().unwrap().clone();
    assert!(
        calls.iter().any(
            |call| call.contains(&format!("api -X PATCH repos/{REPO}/issues/42"))
                && call.contains("body=\n## Problem")
        ),
        "remote issue body was never patched: {calls:#?}"
    );

    let _ = fs::remove_dir_all(local.issues_dir());
    let _ = fs::remove_dir_all(cache.issues_dir());
    server.abort();
}

// HANDWRITE-END
