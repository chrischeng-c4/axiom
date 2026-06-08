// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! End-to-end integration test for `jet build --wasm`.
//!
//! Runs `wasm_build::build` against `examples/counter-demo/`, serves
//! the generated `dist/index.html` with an in-process Rust static
//! server, and asserts the canvas renders text containing "count: 0".
//!
//! Requires node, wasm-pack, and Chromium. Missing prerequisites fail
//! so browser/WASM readiness cannot be claimed by skipped tests.

mod common;

use axum::body::Body;
use axum::extract::{Path as AxumPath, State};
use axum::http::{header, StatusCode};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use jet::browser::{Browser, LaunchOptions};
use jet::build_target::BuildTarget;
use jet::test_runner::{self, RunnerConfig};
use jet::wasm_build::{self, Profile};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn counter_demo_builds_and_renders_on_canvas() {
    common::require_full_wasm_e2e_env();

    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let demo = workspace.join("examples").join("counter-demo");
    // Clean any previous build artifacts so the test exercises the
    // fresh build path.
    let _ = fs::remove_dir_all(demo.join("dist"));

    // 1. Run `jet build --wasm` — call directly into the lib so we
    //    don't shell out to the jet binary.
    jet::wasm_build::build(&demo, std::path::Path::new("dist"))
        .expect("jet build --wasm should succeed");

    // Sanity check outputs exist.
    for f in [
        "index.html",
        "boot.js",
        "jet-host.js",
        "app.js",
        "app_bg.wasm",
    ] {
        let p = demo.join("dist").join(f);
        assert!(p.exists(), "missing {}", p.display());
    }
    let boot = fs::read_to_string(demo.join("dist/boot.js")).unwrap();
    assert!(boot.contains("installJetHost();"));
    assert!(boot.contains("init('./app_bg.wasm')"));

    // 2. Serve dist/ with Rust and drive Chromium to verify rendering.
    let port = spawn_static_server(demo.join("dist")).await;
    let log_file =
        std::env::temp_dir().join(format!("jet-wasm-e2e-{}-{}.log", std::process::id(), port));
    let _ = fs::remove_file(&log_file);

    let tmp = tempfile::tempdir().unwrap();

    let url = format!("http://127.0.0.1:{port}/index.html");
    let spec = format!(
        r#"
import {{ test, expect }} from '@jet/test';

// Sample a broad region and return a compact signature
// "<nonWhite>:<checksum>:<dark>". Sum catches glyph-level changes
// even when non-white pixel count stays similar; `dark` counts
// pixels with every channel < 128 — a proxy for "text pixel".
const SAMPLE_JS = `(() => {{
  const c = document.getElementById('jet-canvas');
  if (!c) return 'NO_CANVAS';
  const ctx = c.getContext('2d');
  const w = Math.min(c.width, 400);
  const h = Math.min(c.height, 100);
  const img = ctx.getImageData(0, 0, w, h);
  let nonWhite = 0, sum = 0, dark = 0;
  for (let i = 0; i < img.data.length; i += 4) {{
    const r = img.data[i], g = img.data[i+1], b = img.data[i+2];
    if (r < 250 || g < 250 || b < 250) nonWhite += 1;
    if (r < 128 && g < 128 && b < 128) dark += 1;
    sum = (sum + r + g + b) | 0;
  }}
  return nonWhite + ':' + (sum >>> 0) + ':' + dark + ':' + w + 'x' + h;
}})()`;

test('counter-demo renders on canvas and increments on click', async ({{ page }}) => {{
  await page.goto({url_json});
  // Wait for WASM to instantiate + first paint.
  await page.waitForTimeout(1500);

  const before = await page.evaluate(SAMPLE_JS);
  if (before === 'NO_CANVAS') throw new Error('canvas element missing');
  const beforeNonWhite = parseInt(before.split(':')[0], 10);
  if (!(beforeNonWhite > 50)) {{
    throw new Error('canvas appears empty on first paint — got ' + before);
  }}

  // Click inside the button via CDP Input.dispatchMouseEvent. The
  // button is drawn at the top-left, default height 24px, full
  // viewport width — (30, 12) lands on the button.
  await page.mouse.click(30, 12);
  // Give the render loop a beat to flush + repaint.
  await page.waitForTimeout(200);

  const after = await page.evaluate(SAMPLE_JS);
  const fs = await import('node:fs/promises');
  await fs.writeFile({log_path_json}, 'before=' + before + ' after=' + after);

  if (before === after) {{
    throw new Error(
      'click produced no visible change — before=' + before + ' after=' + after +
      ' (hit-test / flush / repaint pipeline broken?)'
    );
  }}
}});
"#,
        url_json = serde_json::to_string(&url).unwrap(),
        log_path_json = serde_json::to_string(&log_file.display().to_string()).unwrap(),
    );

    let spec_path = tmp.path().join("wasm_e2e.spec.js");
    fs::write(&spec_path, &spec).unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![jet::test_runner::config::Reporter::Term];
    cfg.workers = 1;
    cfg.headless = true;
    cfg.timeout_ms = 60_000;

    let summary = test_runner::run(cfg)
        .await
        .expect("test runner should complete");

    if let Ok(log) = fs::read_to_string(&log_file) {
        println!("\n[wasm-e2e] sample: {log}");
    }
    assert_eq!(summary.failed, 0, "wasm e2e spec failed: {summary:?}");
    assert_eq!(summary.passed, 1);
}

#[test]
fn webgpu_renderer_build_selects_webgpu_scaffold() {
    common::require_wasm_pack_env();

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_webgpu_fixture(root);

    wasm_build::build_with_profile(root, Path::new("dist"), Profile::Release, BuildTarget::Web)
        .expect("webgpu renderer wasm build should succeed");

    for f in [
        "index.html",
        "boot.js",
        "jet-host.js",
        "app.js",
        "app_bg.wasm",
    ] {
        let p = root.join("dist").join(f);
        assert!(p.exists(), "missing {}", p.display());
    }
    let boot = fs::read_to_string(root.join("dist/boot.js")).unwrap();
    assert!(boot.contains("installJetHost();"));
    assert!(boot.contains("init('./app_bg.wasm')"));

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("dist/jet-target.json")).unwrap())
            .unwrap();
    let features = manifest["build"]["cargo_features"]
        .as_array()
        .expect("cargo_features array");
    assert!(features.iter().any(|f| f == "jet-wasm/webgpu"));
    assert!(features.iter().any(|f| f == "jet-wasm/webgpu-app"));
    assert!(!features.iter().any(|f| f == "jet-wasm/canvas-app"));

    let cargo_toml = fs::read_to_string(root.join(".jet/wasm-build/Cargo.toml")).unwrap();
    assert!(cargo_toml.contains(r#""webgpu""#));
    assert!(cargo_toml.contains(r#""webgpu-app""#));
    assert!(!cargo_toml.contains(r#""canvas-app""#));

    let generated = fs::read_to_string(root.join(".jet/wasm-build/src/lib.rs")).unwrap();
    assert!(generated.contains("jet_wasm::react::webgpu_app::run"));
    assert!(!generated.contains("jet_wasm::react::canvas_app::run"));
}

#[test]
fn wasm_build_bundles_css_side_effect_imports() {
    common::require_wasm_pack_env();

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_css_import_fixture(root);

    wasm_build::build_with_profile(root, Path::new("dist"), Profile::Release, BuildTarget::Web)
        .expect("wasm build should bundle CSS side-effect imports");

    let html = fs::read_to_string(root.join("dist/index.html")).unwrap();
    assert!(html.contains(r#"<link rel="stylesheet" href="./style.css" />"#));
    let css = fs::read_to_string(root.join("dist/style.css")).unwrap();
    assert!(css.contains("/* ./styles.css */"));
    assert!(css.contains("#jet-canvas { background: rgb(1, 2, 3); }"));
}

/// @spec .aw/tech-design/projects/jet/specs/4072.md#unit-test
#[test]
fn wasm_build_compat_lowers_mui_runtime_imports() {
    common::require_wasm_pack_env();

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_mui_compat_fixture(root);

    wasm_build::build_with_profile(root, Path::new("dist"), Profile::Release, BuildTarget::Web)
        .expect("wasm compat build should lower MUI imports into Rust/WASM");

    assert!(root.join("dist/app_bg.wasm").exists());
    assert!(root.join("dist/jet-host.js").exists());
    let generated = fs::read_to_string(root.join(".jet/wasm-build/src/lib.rs")).unwrap();
    assert!(generated.contains("Element::intrinsic(\"button\""));
    assert!(generated.contains("Element::intrinsic(\"input\""));
    assert!(generated.contains("Element::text(\"Create Project\")"));
    assert!(generated.contains("value: Some(\"Ada\".to_string())"));
    assert!(generated.contains("placeholder: Some(\"Name\".to_string())"));
    assert!(generated.contains("input_type: Some(\"checkbox\".to_string())"));
    assert!(generated.contains("checked: Some(true)"));
    assert!(!generated.contains("@mui/material"));
}

/// @spec .aw/tech-design/projects/jet/specs/4072.md#unit-test
#[test]
fn wasm_build_compat_lowers_antd_runtime_imports() {
    common::require_wasm_pack_env();

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_antd_compat_fixture(root);

    wasm_build::build_with_profile(root, Path::new("dist"), Profile::Release, BuildTarget::Web)
        .expect("wasm compat build should lower Ant Design imports into Rust/WASM");

    assert!(root.join("dist/app_bg.wasm").exists());
    assert!(root.join("dist/jet-host.js").exists());
    let generated = fs::read_to_string(root.join(".jet/wasm-build/src/lib.rs")).unwrap();
    assert!(generated.contains("Element::intrinsic(\"button\""));
    assert!(generated.contains("Element::intrinsic(\"input\""));
    assert!(
        generated.contains("Launch Flow"),
        "generated AntD compat fixture:\n{generated}"
    );
    assert!(generated.contains("value: Some(\"Ada\".to_string())"));
    assert!(generated.contains("placeholder: Some(\"Name\".to_string())"));
    assert!(!generated.contains("antd"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn use_effect_fetch_reaches_host_api_from_wasm() {
    common::require_env();

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_css_import_fixture(root);

    wasm_build::build_with_profile(root, Path::new("dist"), Profile::Release, BuildTarget::Web)
        .expect("wasm build with fetch effect should succeed");

    let hits = Arc::new(AtomicUsize::new(0));
    let port = spawn_static_server_with_api(root.join("dist"), hits.clone()).await;
    let url = format!("http://127.0.0.1:{port}/index.html");

    let browser = Browser::launch(LaunchOptions::default())
        .await
        .expect("launch Chromium");
    let page = browser.new_page().await.expect("open page");
    page.goto(&url).await.expect("navigate to fetch-effect app");

    for _ in 0..40 {
        if hits.load(Ordering::SeqCst) > 0 {
            let _ = browser.close().await;
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    let _ = browser.close().await;
    panic!("WASM useEffect fetch did not reach /api/projects");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cue_artifact_studio_dom_wasm_loads_api_and_posts() {
    common::require_env();

    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let root = workspace
        .join("projects")
        .join("cue")
        .join("artifact-studio");
    let out_dir = Path::new(".jet").join("cue-wasm-e2e-dist");
    let _ = fs::remove_dir_all(root.join(&out_dir));

    wasm_build::build_with_profile(&root, &out_dir, Profile::Release, BuildTarget::Web)
        .expect("Cue Artifact Studio DOM wasm build should succeed");

    let post_projects = Arc::new(AtomicUsize::new(0));
    let post_messages = Arc::new(AtomicUsize::new(0));
    let port = spawn_cue_wasm_server(
        root.join(&out_dir),
        post_projects.clone(),
        post_messages.clone(),
    )
    .await;
    let url = format!("http://127.0.0.1:{port}/index.html");

    let browser = Browser::launch(LaunchOptions::default())
        .await
        .expect("launch Chromium");
    let page = browser.new_page().await.expect("open page");
    page.goto(&url).await.expect("navigate to Cue wasm app");

    let text = wait_for_body_text(&page, "Team Request Tracker").await;
    assert!(text.contains("Artifact Studio"), "body text: {text}");
    assert!(text.contains("Request tracker intake"), "body text: {text}");
    assert!(
        text.contains("Create request tracker PRD"),
        "body text: {text}"
    );

    page.evaluate(
        r#"(() => {
            const button = [...document.querySelectorAll('button')]
              .find((node) => node.textContent.includes('New Project'));
            if (!button) throw new Error('New Project button missing');
            button.click();
            return true;
        })()"#,
    )
    .await
    .expect("click New Project");
    wait_for_counter(post_projects.clone(), "POST /api/projects").await;

    page.evaluate(
        r#"(() => {
            const input = document.querySelector('input.jet-input');
            if (!input) throw new Error('composer input missing');
            input.value = 'Create a todo app from WASM';
            input.dispatchEvent(new Event('input', { bubbles: true }));
            const button = [...document.querySelectorAll('button')]
              .find((node) => node.textContent.trim() === 'Send');
            if (!button) throw new Error('Send button missing');
            button.click();
            return true;
        })()"#,
    )
    .await
    .expect("send message");
    wait_for_counter(post_messages.clone(), "POST /api/sessions/:id/messages").await;

    let _ = browser.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webgpu_renderer_reports_runtime_status_when_available() {
    common::require_env();

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_webgpu_fixture(root);

    wasm_build::build_with_profile(root, Path::new("dist"), Profile::Release, BuildTarget::Web)
        .expect("webgpu renderer wasm build should succeed");

    let port = spawn_static_server(root.join("dist")).await;
    let url = format!("http://127.0.0.1:{port}/index.html");

    let mut options = LaunchOptions::default();
    if let Ok(p) = std::env::var("CHROME_PATH") {
        options.executable = Some(std::path::PathBuf::from(p));
    }
    options.args.push("--enable-unsafe-webgpu".to_string());
    let browser = Browser::launch(options).await.expect("launch Chromium");
    let page = browser.new_page().await.expect("open page");
    page.goto(&url).await.expect("navigate to WebGPU app");
    tokio::time::sleep(std::time::Duration::from_millis(2500)).await;

    let probe = page
        .evaluate(
            r#"(() => ({
                gpu: !!navigator.gpu,
                status: window.__jet_webgpu_status ?? null
            }))()"#,
        )
        .await
        .expect("probe WebGPU status");

    if probe.get("gpu").and_then(|v| v.as_bool()) != Some(true) {
        eprintln!("skipping: Chromium launched without navigator.gpu");
        let _ = browser.close().await;
        return;
    }

    let status = probe
        .get("status")
        .and_then(|v| v.as_object())
        .expect("window.__jet_webgpu_status object when WebGPU is available");
    assert_ne!(status.get("phase").and_then(|v| v.as_str()), Some("error"));
    assert_eq!(
        status.get("bridgeMode").and_then(|v| v.as_str()),
        Some("text"),
        "expected WebGPU app to use renderFrameWithText bridge, got {status:?}"
    );
    assert!(
        status.get("frames").and_then(|v| v.as_f64()).unwrap_or(0.0) >= 1.0,
        "expected at least one WebGPU frame, got {status:?}"
    );
    assert!(
        status
            .get("lastCellCount")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            >= 1.0,
        "expected at least one lowered WebGPU cell, got {status:?}"
    );
    assert!(
        status
            .get("lastTextRunCount")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            >= 1.0,
        "expected at least one planned WebGPU text run, got {status:?}"
    );
    // T8 (slice #2191): the encode_text_pass seam must observe at
    // least one glyph instance when text runs are present, proving
    // the bridge planned glyphs and the renderer encoded the text
    // pass against them.
    assert!(
        status
            .get("lastTextGlyphCount")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            >= 1.0,
        "expected at least one planned WebGPU text glyph, got {status:?}"
    );

    let _ = browser.close().await;
}

fn write_css_import_fixture(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("jet.config.toml"),
        r#"
[wasm]
entry = "src/App.tsx"
root_component = "App"
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/styles.css"),
        "#jet-canvas { background: rgb(1, 2, 3); }",
    )
    .unwrap();
    fs::write(
        root.join("src/App.tsx"),
        r#"
import React, { useEffect } from 'react';
import './styles.css';

interface AppProps {}

export function App({}: AppProps) {
  useEffect(() => {
    fetch('/api/projects');
  }, []);

  return (
    <button id="styled">
      styled
    </button>
  );
}
"#,
    )
    .unwrap();
}

fn write_mui_compat_fixture(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("index.html"),
        r#"<!doctype html>
<html>
  <head><title>MUI compat</title></head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"#,
    )
    .unwrap();
    fs::write(
        root.join("jet.config.toml"),
        r#"
[wasm]
entry = "src/main.tsx"
root_component = "App"
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/main.tsx"),
        r#"
import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './App.tsx';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(<App />);
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/App.tsx"),
        r#"
import React from 'react';
import AddRoundedIcon from '@mui/icons-material/AddRounded';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Checkbox from '@mui/material/Checkbox';
import CssBaseline from '@mui/material/CssBaseline';
import TextField from '@mui/material/TextField';
import { ThemeProvider } from '@mui/material/styles';
import { fetchProjects } from './api';

export function App() {
  return (
    <ThemeProvider theme={{}}>
      <CssBaseline />
      <Box id="root">
        <Button startIcon={<AddRoundedIcon />}>Create Project</Button>
        <TextField id="project-name" defaultValue="Ada" placeholder="Name" />
        <Checkbox id="accepted" type="checkbox" defaultChecked aria-label="Accept" />
      </Box>
    </ThemeProvider>
  );
}
"#,
    )
    .unwrap();
}

fn write_antd_compat_fixture(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("jet.config.toml"),
        r#"
[wasm]
entry = "src/App.tsx"
root_component = "App"
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/App.tsx"),
        r#"
import React from 'react';
import { Button, Input } from 'antd';

export function App() {
  return (
    <form id="launch-form">
      <Button id="launch" className="ant-btn ant-btn-primary">
        Launch Flow
      </Button>
      <Input id="flow-name" className="ant-input" defaultValue="Ada" placeholder="Name" />
    </form>
  );
}
"#,
    )
    .unwrap();
}

fn write_webgpu_fixture(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("jet.config.toml"),
        r#"
[wasm]
entry = "src/App.tsx"
root_component = "App"
renderer = "web-gpu"
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/App.tsx"),
        r#"
interface AppProps {}

export function App({}: AppProps) {
  return (
    <button id="webgpu">
      webgpu
    </button>
  );
}
"#,
    )
    .unwrap();
}

async fn spawn_static_server(dist: PathBuf) -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let app = Router::new()
        .route("/", get(handle_index))
        .route("/{*path}", get(handle_static))
        .with_state(Arc::new(dist));

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    port
}

#[derive(Clone)]
struct ApiStaticState {
    dist: Arc<PathBuf>,
    hits: Arc<AtomicUsize>,
}

async fn spawn_static_server_with_api(dist: PathBuf, hits: Arc<AtomicUsize>) -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let app = Router::new()
        .route("/api/projects", get(handle_api_projects))
        .route("/", get(handle_api_index))
        .route("/{*path}", get(handle_api_static))
        .with_state(ApiStaticState {
            dist: Arc::new(dist),
            hits,
        });

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    port
}

#[derive(Clone)]
struct CueWasmState {
    dist: Arc<PathBuf>,
    post_projects: Arc<AtomicUsize>,
    post_messages: Arc<AtomicUsize>,
}

async fn spawn_cue_wasm_server(
    dist: PathBuf,
    post_projects: Arc<AtomicUsize>,
    post_messages: Arc<AtomicUsize>,
) -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let app = Router::new()
        .route(
            "/api/projects",
            get(handle_cue_api_projects).post(handle_cue_post_project),
        )
        .route(
            "/api/sessions/{session_id}/messages",
            post(handle_cue_post_message),
        )
        .route("/", get(handle_cue_wasm_index))
        .route("/{*path}", get(handle_cue_wasm_static))
        .with_state(CueWasmState {
            dist: Arc::new(dist),
            post_projects,
            post_messages,
        });

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    port
}

async fn handle_cue_api_projects() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(cue_projects_json()))
        .unwrap()
}

async fn handle_cue_post_project(State(state): State<CueWasmState>) -> Response {
    state.post_projects.fetch_add(1, Ordering::SeqCst);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"project":{"id":"todo-app-project"}}"#))
        .unwrap()
}

async fn handle_cue_post_message(
    AxumPath(_session_id): AxumPath<String>,
    State(state): State<CueWasmState>,
) -> Response {
    state.post_messages.fetch_add(1, Ordering::SeqCst);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"ok":true}"#))
        .unwrap()
}

async fn handle_cue_wasm_index(State(state): State<CueWasmState>) -> Response {
    serve_file(&state.dist.join("index.html"))
}

async fn handle_cue_wasm_static(
    AxumPath(path): AxumPath<String>,
    State(state): State<CueWasmState>,
) -> Response {
    let rel = PathBuf::from(&path);
    for comp in rel.components() {
        if matches!(comp, std::path::Component::ParentDir) {
            return not_found();
        }
    }
    serve_file(&state.dist.join(rel))
}

fn cue_projects_json() -> &'static str {
    r#"{
      "projects": [
        {
          "id": "team-request-tracker",
          "name": "Team Request Tracker",
          "owner": "Operations",
          "status": "needs-review",
          "next_action": "Review PRD",
          "summary": "WorkItem accepted; PRD is waiting for owner review.",
          "active_session_id": "session-request-tracker",
          "sessions": [
            {
              "id": "session-request-tracker",
              "project_id": "team-request-tracker",
              "title": "Request tracker intake",
              "messages": [
                { "id": "m1", "speaker": "owner", "body": "We need an internal request tracker." },
                { "id": "m2", "speaker": "cue", "body": "WorkItem accepted; next step is PRD review." }
              ]
            }
          ],
          "workitems": [
            {
              "id": "request-tracker-prd",
              "project_id": "team-request-tracker",
              "title": "Create request tracker PRD",
              "route": "prompt-to-PRD",
              "target": "PRD",
              "state": "accepted",
              "progress": 100,
              "next_action": "Review PRD",
              "blockers": [],
              "workflow_plan": [],
              "qc_status": "pass",
              "qc_checks": []
            }
          ]
        }
      ]
    }"#
}

async fn wait_for_body_text(page: &jet::browser::Page, expected: &str) -> String {
    for _ in 0..40 {
        let value = page
            .evaluate("document.body.innerText")
            .await
            .expect("read body text");
        let text = value.as_str().unwrap_or_default().to_string();
        if text.contains(expected) {
            return text;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
    page.evaluate("document.body.innerText")
        .await
        .expect("read final body text")
        .as_str()
        .unwrap_or_default()
        .to_string()
}

async fn wait_for_counter(counter: Arc<AtomicUsize>, label: &str) {
    for _ in 0..40 {
        if counter.load(Ordering::SeqCst) > 0 {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
    panic!("{label} was not observed");
}

async fn handle_api_projects(State(state): State<ApiStaticState>) -> Response {
    state.hits.fetch_add(1, Ordering::SeqCst);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"projects":[]}"#))
        .unwrap()
}

async fn handle_api_index(State(state): State<ApiStaticState>) -> Response {
    serve_file(&state.dist.join("index.html"))
}

async fn handle_api_static(
    AxumPath(path): AxumPath<String>,
    State(state): State<ApiStaticState>,
) -> Response {
    let rel = PathBuf::from(&path);
    for comp in rel.components() {
        if matches!(comp, std::path::Component::ParentDir) {
            return not_found();
        }
    }
    serve_file(&state.dist.join(rel))
}

async fn handle_index(State(dist): State<Arc<PathBuf>>) -> Response {
    serve_file(&dist.join("index.html"))
}

async fn handle_static(
    AxumPath(path): AxumPath<String>,
    State(dist): State<Arc<PathBuf>>,
) -> Response {
    let rel = PathBuf::from(&path);
    for comp in rel.components() {
        if matches!(comp, std::path::Component::ParentDir) {
            return not_found();
        }
    }
    serve_file(&dist.join(rel))
}

fn serve_file(path: &Path) -> Response {
    let body = match fs::read(path) {
        Ok(body) => body,
        Err(_) => return not_found(),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type_for(path))
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(body))
        .unwrap()
}

fn not_found() -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from("404 Not Found"))
        .unwrap()
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("css") => "text/css; charset=utf-8",
        Some("json") | Some("map") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}
// CODEGEN-END
