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

#[path = "../common/mod.rs"]
mod common;

use axum::body::Body;
use axum::extract::{Path as AxumPath, State};
use axum::http::{header, StatusCode};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use jet::browser::{Browser, LaunchOptions};
use jet::build_target::BuildTarget;
use jet::wasm_build::{self, manifest as wasm_manifest, Profile};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

static WASM_PACK_BUILD_LOCK: Mutex<()> = Mutex::new(());

fn wasm_pack_build_lock() -> MutexGuard<'static, ()> {
    WASM_PACK_BUILD_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn build_wasm_serialized_with_profile(
    root: &Path,
    out_dir: &Path,
    profile: Profile,
) -> anyhow::Result<()> {
    let _guard = wasm_pack_build_lock();
    wasm_build::build_with_profile(root, out_dir, profile, BuildTarget::Web)
}

fn build_wasm_serialized(root: &Path, out_dir: &Path) -> anyhow::Result<()> {
    build_wasm_serialized_with_profile(root, out_dir, Profile::Release)
}

fn build_default_wasm_serialized(root: &Path, out_dir: &Path) -> anyhow::Result<()> {
    let _guard = wasm_pack_build_lock();
    jet::wasm_build::build(root, out_dir)
}

const WEBGPU_VISUAL_PROBE_JS: &str = r#"(() => {
    function canvasVisualProbe() {
        const canvas = document.querySelector('canvas#jet-canvas');
        if (!canvas) return { error: 'missing-canvas' };
        const sourceW = Math.max(1, Math.min(canvas.width || canvas.clientWidth || 1, 1024));
        const sourceH = Math.max(1, Math.min(canvas.height || canvas.clientHeight || 1, 1024));
        const sample = document.createElement('canvas');
        sample.width = 32;
        sample.height = 32;
        const ctx = sample.getContext('2d', { willReadFrequently: true });
        if (!ctx) return { error: 'missing-2d-context' };

        try {
            ctx.drawImage(canvas, 0, 0, sourceW, sourceH, 0, 0, 32, 32);
        } catch (error) {
            return { error: String(error) };
        }

        const data = ctx.getImageData(0, 0, 32, 32).data;
        let nonTransparent = 0;
        let nonWhite = 0;
        let nonBlack = 0;
        const buckets = new Set();
        const luma = [];
        for (let i = 0; i < data.length; i += 4) {
            const r = data[i];
            const g = data[i + 1];
            const b = data[i + 2];
            const a = data[i + 3];
            if (a > 0) nonTransparent += 1;
            if (a > 0 && (r < 250 || g < 250 || b < 250)) nonWhite += 1;
            if (a > 0 && (r > 5 || g > 5 || b > 5)) nonBlack += 1;
            buckets.add(`${r >> 5}:${g >> 5}:${b >> 5}:${a >> 5}`);
            luma.push((299 * r + 587 * g + 114 * b) / 1000);
        }

        const blockLuma = [];
        for (let by = 0; by < 8; by += 1) {
            for (let bx = 0; bx < 8; bx += 1) {
                let sum = 0;
                for (let y = 0; y < 4; y += 1) {
                    for (let x = 0; x < 4; x += 1) {
                        sum += luma[(by * 4 + y) * 32 + (bx * 4 + x)];
                    }
                }
                blockLuma.push(sum / 16);
            }
        }
        const avg = blockLuma.reduce((acc, value) => acc + value, 0) / blockLuma.length;
        let hash = '';
        let ones = 0;
        for (let i = 0; i < blockLuma.length; i += 4) {
            let nibble = 0;
            for (let j = 0; j < 4; j += 1) {
                const bit = blockLuma[i + j] >= avg ? 1 : 0;
                ones += bit;
                nibble = (nibble << 1) | bit;
            }
            hash += nibble.toString(16);
        }

        return {
            width: canvas.width,
            height: canvas.height,
            clientWidth: canvas.clientWidth,
            clientHeight: canvas.clientHeight,
            sourceW,
            sourceH,
            nonTransparent,
            nonWhite,
            nonBlack,
            uniqueBuckets: buckets.size,
            averageLuma: avg,
            hash,
            hashOnes: ones
        };
    }

    return {
        gpu: !!navigator.gpu,
        status: window.__jet_webgpu_status ?? null,
        console: window.__jet_console ?? [],
        visualProbe: canvasVisualProbe(),
        canvasCount: document.querySelectorAll('canvas#jet-canvas').length,
        domCellCount: document.querySelectorAll('#large-grid, button, span').length
    };
})()"#;

const WEBGPU_CONSOLE_CAPTURE_JS: &str = r#"(() => {
    const entries = [];
    Object.defineProperty(window, '__jet_console', {
        value: entries,
        configurable: true
    });
    for (const level of ['log', 'warn', 'error']) {
        const original = console[level].bind(console);
        console[level] = (...args) => {
            entries.push({ level, text: args.map((arg) => {
                try {
                    return typeof arg === 'string' ? arg : JSON.stringify(arg);
                } catch (_) {
                    return String(arg);
                }
            }).join(' ') });
            return original(...args);
        };
    }
    window.addEventListener('error', (event) => {
        entries.push({ level: 'error', text: event.message || String(event.error) });
    });
    window.addEventListener('unhandledrejection', (event) => {
        entries.push({ level: 'error', text: String(event.reason) });
    });
})()"#;

const WEBGPU_SETTLE_TWO_RAF_JS: &str = r#"new Promise((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve(null)));
})"#;

fn webgpu_launch_options() -> LaunchOptions {
    let mut options = LaunchOptions::default();
    if let Ok(p) = std::env::var("CHROME_PATH") {
        options.executable = Some(std::path::PathBuf::from(p));
    }
    options.args.push("--enable-unsafe-webgpu".to_string());
    options
}

async fn wait_for_webgpu_rendered(page: &jet::browser::Page, context: &str) -> Option<Value> {
    let mut probe = Value::Null;
    for _ in 0..80 {
        probe = page
            .evaluate(WEBGPU_VISUAL_PROBE_JS)
            .await
            .unwrap_or_else(|err| panic!("{context}: probe WebGPU status failed: {err}"));
        if probe.get("gpu").and_then(|v| v.as_bool()) != Some(true) {
            return None;
        }
        let phase = probe
            .get("status")
            .and_then(|v| v.get("phase"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if phase == "error" {
            panic!("{context}: WebGPU runtime reported error: {probe:?}");
        }
        if phase == "rendered" {
            return Some(probe);
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
    panic!("{context}: WebGPU runtime did not reach rendered phase: {probe:?}");
}

fn assert_visible_webgpu_screenshot(bytes: &[u8], context: &str) -> Value {
    let probe = common::react_oracle::screenshot_visual_probe_from_png(bytes);
    assert!(
        probe
            .get("foregroundCount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 256
            && probe.get("nonWhite").and_then(|v| v.as_u64()).unwrap_or(0) > 0
            && probe
                .get("uniqueBuckets")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                > 1,
        "{context}: expected visible WebGPU screenshot, got {probe:?}"
    );
    probe
}

fn webgpu_status_frames(probe: &Value) -> f64 {
    probe
        .get("status")
        .and_then(|v| v.get("frames"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
}

async fn dispatch_cdp_click(page: &jet::browser::Page, x: f64, y: f64) {
    for (event_type, button, buttons) in [
        ("mouseMoved", "none", 0_u64),
        ("mousePressed", "left", 1_u64),
        ("mouseReleased", "left", 0_u64),
    ] {
        let mut params = serde_json::json!({
            "type": event_type,
            "x": x,
            "y": y,
            "buttons": buttons,
        });
        if button != "none" {
            params["button"] = Value::String(button.to_string());
            params["clickCount"] = Value::from(1);
        }
        page.session()
            .send("Input.dispatchMouseEvent", params)
            .await
            .unwrap_or_else(|err| panic!("dispatch CDP click event {event_type}: {err}"));
    }
}

fn dom_text_summary_expr(expected: &str) -> String {
    let expected = serde_json::to_string(expected).expect("expected text serializes");
    format!(
        r#"
(() => {{
  const text = (document.body?.innerText || '').replace(/\s+/g, ' ').trim();
  return {{
    ok: text.includes({expected}),
    text,
    buttonCount: document.querySelectorAll('button').length,
    canvasCount: document.querySelectorAll('canvas#jet-canvas').length,
    status: window.__jet_webgpu_status ?? null
  }};
}})()
"#
    )
}

async fn wait_for_dom_text(page: &jet::browser::Page, expected: &str, context: &str) -> Value {
    let expr = dom_text_summary_expr(expected);
    let mut summary = Value::Null;
    for _ in 0..80 {
        summary = page
            .evaluate(&expr)
            .await
            .unwrap_or_else(|err| panic!("{context}: DOM summary failed: {err}"));
        if summary.get("ok").and_then(|v| v.as_bool()) == Some(true) {
            return summary;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
    panic!("{context}: DOM never contained {expected:?}: {summary:?}");
}

fn dom_click_button_expr(label: &str) -> String {
    let label = serde_json::to_string(label).expect("button label serializes");
    format!(
        r#"
(() => {{
  const button = Array.from(document.querySelectorAll('button'))
    .find((candidate) => (candidate.textContent || '').includes({label}));
  if (!button) return {{ ok: false, reason: 'missing button', label: {label} }};
  button.click();
  return {{ ok: true, text: button.textContent || '' }};
}})()
"#
    )
}

async fn click_dom_button(page: &jet::browser::Page, label: &str, context: &str) {
    let clicked = page
        .evaluate(&dom_click_button_expr(label))
        .await
        .unwrap_or_else(|err| panic!("{context}: DOM click failed: {err}"));
    assert!(
        clicked.get("ok").and_then(|v| v.as_bool()) == Some(true),
        "{context}: DOM button click failed: {clicked:?}"
    );
}

#[tokio::test]
async fn counter_demo_builds_and_updates_on_webgpu_canvas() {
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
    build_default_wasm_serialized(&demo, std::path::Path::new("dist"))
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
    let url = format!("http://127.0.0.1:{port}/index.html");
    let browser = Browser::launch(webgpu_launch_options())
        .await
        .expect("launch Chromium with WebGPU");
    let page = browser.new_page().await.expect("open page");
    page.add_init_script(WEBGPU_CONSOLE_CAPTURE_JS)
        .await
        .expect("install console capture");
    page.goto(&url).await.expect("navigate to counter wasm app");

    let Some(before_probe) = wait_for_webgpu_rendered(&page, "counter demo").await else {
        eprintln!("skipping: Chromium launched without navigator.gpu");
        let _ = browser.close().await;
        return;
    };
    page.evaluate(WEBGPU_SETTLE_TWO_RAF_JS)
        .await
        .expect("settle initial WebGPU paint");
    let before_png = page
        .screenshot()
        .await
        .expect("capture counter before click");
    let before_visual = assert_visible_webgpu_screenshot(&before_png, "counter before click");
    let before_frames = webgpu_status_frames(&before_probe);

    dispatch_cdp_click(&page, 30.0, 12.0).await;

    let mut after_probe = Value::Null;
    for _ in 0..40 {
        after_probe = page
            .evaluate(WEBGPU_VISUAL_PROBE_JS)
            .await
            .expect("probe counter after click");
        if webgpu_status_frames(&after_probe) > before_frames {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    page.evaluate(WEBGPU_SETTLE_TWO_RAF_JS)
        .await
        .expect("settle post-click WebGPU paint");
    let after_png = page
        .screenshot()
        .await
        .expect("capture counter after click");
    let after_visual = assert_visible_webgpu_screenshot(&after_png, "counter after click");

    let _ = browser.close().await;
    assert!(
        webgpu_status_frames(&after_probe) > before_frames,
        "counter click did not trigger a new WebGPU frame: before={before_probe:?} after={after_probe:?}"
    );
    assert_ne!(
        before_png, after_png,
        "counter click produced no visible screenshot change: before={before_visual:?} after={after_visual:?}"
    );
}

#[test]
fn wasm_build_selects_webgpu_scaffold_by_default() {
    common::require_wasm_pack_env();

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_webgpu_fixture(root);
    fs::create_dir_all(root.join("dist")).unwrap();
    fs::write(root.join("dist/main.stale-dom.js"), "stale dom bundle").unwrap();
    fs::write(root.join("dist/main.stale-dom.js.map"), "{}").unwrap();
    fs::write(root.join("dist/stale.txt"), "stale output").unwrap();

    build_wasm_serialized(root, Path::new("dist"))
        .expect("default wasm build should use WebGPU scaffold");

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
    let mut wrapper_js = String::new();
    for file in ["app.js", "boot.js", "jet-host.js"] {
        wrapper_js.push_str(&fs::read_to_string(root.join("dist").join(file)).unwrap());
        wrapper_js.push('\n');
    }
    for marker in [
        "wasm-owned-app-marker",
        "ReactDOM",
        "createRoot",
        "@mui/material",
        "MUI visual table fixture",
        "cell 9999",
    ] {
        assert!(
            !wrapper_js.contains(marker),
            "WASM build must keep app/domain/render marker {marker:?} out of wrapper JS"
        );
    }
    assert!(
        !root.join("dist/main.stale-dom.js").exists()
            && !root.join("dist/main.stale-dom.js.map").exists()
            && !root.join("dist/stale.txt").exists(),
        "WASM build must clean stale DOM/dev-server artifacts from dist/"
    );

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("dist/jet-target.json")).unwrap())
            .unwrap();
    let features = manifest["build"]["cargo_features"]
        .as_array()
        .expect("cargo_features array");
    assert_eq!(
        features,
        &[
            serde_json::Value::String("jet-wasm/react".to_string()),
            serde_json::Value::String("jet-wasm/webgpu".to_string()),
            serde_json::Value::String("jet-wasm/webgpu-app".to_string()),
        ]
    );
    assert_eq!(
        manifest["build"]["tsx_lowering"].as_str(),
        Some(wasm_manifest::TSX_LOWERING_STRICT),
        "simple fixture must record strict TSX lowering in jet-target.json: {manifest}"
    );

    let cargo_toml = fs::read_to_string(root.join(".jet/wasm-build/Cargo.toml")).unwrap();
    assert!(cargo_toml.contains(r#"features = ["react", "webgpu", "webgpu-app"]"#));

    let generated = fs::read_to_string(root.join(".jet/wasm-build/src/lib.rs")).unwrap();
    assert!(
        generated.contains("wasm-owned-app-marker"),
        "lowered Rust/WASM source must own the app marker, not wrapper JS"
    );
    assert_eq!(
        generated
            .matches("jet_wasm::react::webgpu_app::run")
            .count(),
        1
    );
}

#[test]
fn wasm_build_bundles_css_side_effect_imports() {
    common::require_wasm_pack_env();

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_css_import_fixture(root);

    build_wasm_serialized(root, Path::new("dist"))
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

    build_wasm_serialized(root, Path::new("dist"))
        .expect("wasm compat build should lower MUI imports into Rust/WASM");

    assert!(root.join("dist/app_bg.wasm").exists());
    assert!(root.join("dist/jet-host.js").exists());
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("dist/jet-target.json")).unwrap())
            .unwrap();
    assert_eq!(
        manifest["build"]["tsx_lowering"].as_str(),
        Some(wasm_manifest::TSX_LOWERING_COMPATIBILITY),
        "MUI compat fixture must record compatibility lowering in jet-target.json: {manifest}"
    );
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

    build_wasm_serialized(root, Path::new("dist"))
        .expect("wasm compat build should lower Ant Design imports into Rust/WASM");

    assert!(root.join("dist/app_bg.wasm").exists());
    assert!(root.join("dist/jet-host.js").exists());
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("dist/jet-target.json")).unwrap())
            .unwrap();
    assert_eq!(
        manifest["build"]["tsx_lowering"].as_str(),
        Some(wasm_manifest::TSX_LOWERING_STRICT),
        "Ant Design runtime fixture must record strict lowering in jet-target.json: {manifest}"
    );
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

    build_wasm_serialized(root, Path::new("dist"))
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

    build_wasm_serialized_with_profile(&root, &out_dir, Profile::Dev)
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

    let browser = Browser::launch(webgpu_launch_options())
        .await
        .expect("launch Chromium");
    let page = browser.new_page().await.expect("open page");
    page.add_init_script(WEBGPU_CONSOLE_CAPTURE_JS)
        .await
        .expect("install console capture");
    page.goto(&url).await.expect("navigate to Cue wasm app");

    let initial_summary = wait_for_dom_text(
        &page,
        "Team Request Tracker",
        "Cue Artifact Studio initial fetch repaint",
    )
    .await;
    let initial_text = initial_summary
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        initial_text.contains("Artifact Studio"),
        "debug tree text: {initial_text}"
    );
    assert!(
        initial_text.contains("Request tracker intake"),
        "debug tree text: {initial_text}"
    );
    assert!(
        initial_text.contains("Create request tracker PRD"),
        "debug tree text: {initial_text}"
    );

    assert!(
        initial_summary
            .get("buttonCount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            >= 1,
        "Cue DOM renderer should expose clickable buttons: {initial_summary:?}"
    );
    assert_eq!(
        initial_summary.get("canvasCount").and_then(|v| v.as_u64()),
        Some(0),
        "Cue DOM renderer should not mount the WebGPU canvas: {initial_summary:?}"
    );
    click_dom_button(&page, "New Project", "Cue Artifact Studio").await;
    wait_for_counter(post_projects.clone(), "POST /api/projects").await;

    let summary = wait_for_dom_text(
        &page,
        "Team Request Tracker",
        "Cue Artifact Studio refreshed",
    )
    .await;
    let text = summary.get("text").and_then(|v| v.as_str()).unwrap_or("");
    assert!(text.contains("Artifact Studio"), "debug tree text: {text}");
    assert!(
        text.contains("Request tracker intake"),
        "debug tree text: {text}"
    );
    assert!(
        text.contains("Create request tracker PRD"),
        "debug tree text: {text}"
    );
    assert!(
        summary
            .get("buttonCount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            >= 1,
        "Cue DOM renderer should remain interactive after POST: {summary:?}"
    );

    let _ = browser.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webgpu_renderer_reports_runtime_status_and_visual_probe_when_available() {
    common::require_env();

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_webgpu_large_table_fixture(root, 10_000);

    build_wasm_serialized(root, Path::new("dist"))
        .expect("default WebGPU wasm build should succeed");

    let port = spawn_static_server(root.join("dist")).await;
    let url = format!("http://127.0.0.1:{port}/index.html");

    let mut options = LaunchOptions::default();
    if let Ok(p) = std::env::var("CHROME_PATH") {
        options.executable = Some(std::path::PathBuf::from(p));
    }
    options.args.push("--enable-unsafe-webgpu".to_string());
    let browser = Browser::launch(options).await.expect("launch Chromium");
    let page = browser.new_page().await.expect("open page");
    page.add_init_script(WEBGPU_CONSOLE_CAPTURE_JS)
        .await
        .expect("install console capture");
    page.goto(&url).await.expect("navigate to WebGPU app");

    let mut probe = serde_json::Value::Null;
    for _ in 0..80 {
        probe = page
            .evaluate(WEBGPU_VISUAL_PROBE_JS)
            .await
            .expect("probe WebGPU status");
        if probe.get("gpu").and_then(|v| v.as_bool()) != Some(true) {
            break;
        }
        let phase = probe
            .get("status")
            .and_then(|v| v.get("phase"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if matches!(phase, "mounted" | "rendered" | "error") {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    if probe.get("gpu").and_then(|v| v.as_bool()) != Some(true) {
        eprintln!("skipping: Chromium launched without navigator.gpu");
        let _ = browser.close().await;
        return;
    }

    let status = probe
        .get("status")
        .and_then(|v| v.as_object())
        .expect("window.__jet_webgpu_status object when WebGPU is available");
    let visual_probe = probe
        .get("visualProbe")
        .and_then(|v| v.as_object())
        .expect("canvas visual probe object when WebGPU is available");
    assert!(
        visual_probe.get("error").is_none(),
        "canvas visual probe failed: {visual_probe:?}"
    );
    assert_eq!(
        probe.get("canvasCount").and_then(|v| v.as_u64()),
        Some(1),
        "expected exactly one wrapper canvas, got {probe:?}"
    );
    assert_eq!(
        probe.get("domCellCount").and_then(|v| v.as_u64()),
        Some(0),
        "large table cells must not be materialized as DOM nodes; got {probe:?}"
    );
    assert_ne!(status.get("phase").and_then(|v| v.as_str()), Some("error"));
    assert_eq!(
        status.get("bridgeMode").and_then(|v| v.as_str()),
        Some("text"),
        "expected WebGPU app to use renderFrameWithText bridge, got {probe:?}"
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
            >= 10_000.0,
        "expected the large table to lower at least 10k WebGPU cells, got {status:?}"
    );
    assert!(
        status
            .get("lastTextRunCount")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            >= 10_000.0,
        "expected the large table to plan at least 10k WebGPU text runs, got {status:?}"
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
    assert_eq!(
        status.get("textAtlasMode").and_then(|v| v.as_str()),
        Some("glyph-atlas"),
        "expected WebGPU text to sample a real glyph atlas, got {status:?}"
    );
    assert!(
        status
            .get("lastTextAtlasUploadCount")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            >= 1.0,
        "expected at least one real glyph atlas upload, got {status:?}"
    );
    assert!(
        status
            .get("lastTextAtlasWidth")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            > 1.0
            && status
                .get("lastTextAtlasHeight")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
                > 1.0
            && status
                .get("lastTextAtlasNonZeroAlphaCount")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
                > 0.0,
        "expected non-placeholder glyph atlas dimensions and alpha pixels, got {status:?}"
    );

    page.evaluate(WEBGPU_SETTLE_TWO_RAF_JS)
        .await
        .expect("wait for browser paint after WebGPU present");
    let final_probe = page
        .evaluate(WEBGPU_VISUAL_PROBE_JS)
        .await
        .expect("probe WebGPU status after paint");
    let screenshot = page.screenshot().await.expect("capture WebGPU screenshot");
    let screenshot_probe = common::react_oracle::screenshot_visual_probe_from_png(&screenshot);
    let visual_diagnostics = serde_json::json!({
        "runtime": final_probe,
        "screenshot": screenshot_probe,
    });
    assert!(
        screenshot_probe
            .get("nonTransparent")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0,
        "expected screenshot probe to contain visible alpha, got {visual_diagnostics}"
    );
    assert!(
        screenshot_probe
            .get("nonWhite")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0,
        "expected screenshot probe to contain non-white pixels, got {visual_diagnostics}"
    );
    assert!(
        screenshot_probe
            .get("nonBlack")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0,
        "expected screenshot probe to contain non-black pixels, got {visual_diagnostics}"
    );
    assert!(
        screenshot_probe
            .get("uniqueBuckets")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 1,
        "expected screenshot probe to contain more than one color bucket, got {visual_diagnostics}"
    );
    assert!(
        screenshot_probe
            .get("foregroundCount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 256,
        "expected screenshot foreground pixels from rendered cells/text, got {visual_diagnostics}"
    );
    let hash = screenshot_probe
        .get("hash")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let hash_ones = screenshot_probe
        .get("hashOnes")
        .and_then(|v| v.as_u64())
        .unwrap_or(64);
    assert!(
        !hash.is_empty() && hash_ones > 0 && hash_ones < 64,
        "expected non-solid screenshot perceptual hash, got {visual_diagnostics}"
    );

    let _ = browser.close().await;
}

fn write_css_import_fixture(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("jet.toml"),
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
        root.join("jet.toml"),
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
        root.join("jet.toml"),
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
        root.join("jet.toml"),
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
interface AppProps {}

export function App({}: AppProps) {
  return (
    <button id="wasm-owned-app-marker">
      wasm-owned-app-marker
    </button>
  );
}
"#,
    )
    .unwrap();
}

fn write_webgpu_large_table_fixture(root: &Path, cell_count: usize) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("jet.toml"),
        r#"
[wasm]
entry = "src/App.tsx"
root_component = "App"
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/App.tsx"),
        format!(
            r#"
interface AppProps {{}}

export function App({{}}: AppProps) {{
  return (
    <div id="large-grid">
      {{[...Array({cell_count})].map((_, idx) => (
        <button>
          {{idx}}
        </button>
      ))}}
    </div>
  );
}}
"#,
        ),
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
