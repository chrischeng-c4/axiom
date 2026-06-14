// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! End-to-end smoke test for `jet browser` debugging commands.
//!
//! Flow:
//! 1. `jet build --wasm --debug` on examples/counter-demo.
//! 2. Spawn `jet dev --wasm` serving dist/ on a free port.
//! 3. Call `browser_cli::prepare_session(url)` — boots Chromium,
//!    writes the session file. Holds the Browser for the test's
//!    lifetime so we can close cleanly.
//! 4. Exercise each subcommand against the live app:
//!    - `tree element` / `tree layout` / `tree fiber`
//!    - `hooks 0` (the counter fiber)
//!    - `highlight 0` + `highlight --clear`
//!    - `frame`
//!    - `eval "window.__jet_debug.forceRerender()"`
//!
//! Requires `wasm-pack` + Chromium. Missing prerequisites fail so
//! browser/WASM readiness cannot be claimed by skipped tests.

#[path = "../common/mod.rs"]
mod common;

use jet::browser_cli;
use jet::wasm_build::manifest as wasm_manifest;
use jet::wasm_build::{self, Profile};
use jet::wasm_dev::{self, DevOptions};
use std::fs;

fn assert_gpu_timing_status_observable(webgpu_status: &serde_json::Value) {
    let timing_enabled = webgpu_status
        .get("gpuTimingEnabled")
        .and_then(|v| v.as_bool())
        .expect("browser capture must include gpuTimingEnabled boolean");
    let sample_ready = webgpu_status
        .get("gpuTimingSampleReady")
        .and_then(|v| v.as_bool())
        .expect("browser capture must include gpuTimingSampleReady boolean");
    let last_ms = webgpu_status
        .get("lastFrameGpuMs")
        .expect("browser capture must include lastFrameGpuMs field");
    if !timing_enabled {
        assert!(
            !sample_ready,
            "GPU timing cannot be sample-ready when TIMESTAMP_QUERY is disabled: {webgpu_status:?}"
        );
    }
    if sample_ready {
        let ms = last_ms
            .as_f64()
            .expect("lastFrameGpuMs must be numeric when gpuTimingSampleReady=true");
        assert!(
            ms.is_finite() && ms >= 0.0,
            "lastFrameGpuMs must be a finite non-negative duration: {webgpu_status:?}"
        );
    } else {
        assert!(
            last_ms.is_null(),
            "lastFrameGpuMs must be null until a GPU timing sample is ready: {webgpu_status:?}"
        );
    }
}

async fn free_port() -> u16 {
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    l.local_addr().unwrap().port()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn browser_cli_drives_debug_bridge_end_to_end() {
    common::require_env();

    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let demo = workspace.join("examples").join("counter-demo");
    let _ = fs::remove_dir_all(demo.join("dist"));
    // Keep .jet/wasm-build across runs — the incremental cache saves
    // ~30s per rerun. We just re-run wasm-pack.

    // 1. Debug-profile build so window.__jet_debug is live.
    wasm_build::build_with_profile(
        &demo,
        std::path::Path::new("dist"),
        Profile::Dev,
        jet::build_target::BuildTarget::Web,
    )
    .expect("jet build --wasm --debug");
    assert!(demo.join("dist/app_bg.wasm").exists());

    // 2. Serve dist/ — give the dev server its own port + own root
    // dir (the counter-demo dir itself).
    let port = free_port().await;
    let serve_root = demo.clone();
    let serve = tokio::spawn(async move {
        wasm_dev::serve(
            &serve_root,
            DevOptions {
                host: "127.0.0.1".to_string(),
                port,
                debug: true,
            },
        )
        .await
    });
    // Wait for the server to accept a connection.
    let url = format!("http://127.0.0.1:{port}/");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap();
    let ready = common::wait_for_http_ready(&client, &url).await;
    assert!(ready, "wasm_dev never came up at {url}");

    // 3. Launch Chromium + write session.
    let browser = browser_cli::prepare_session(&demo, &url)
        .await
        .expect("prepare_session");

    // Give wasm-bindgen init a beat to finish (start fn runs + mounts
    // + registers window.__jet_debug).
    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

    // 4. Exercise the CLI commands against the session. We assert by
    // side-effect: each command prints to stdout — we capture those
    // for later assertions by calling the same underlying fn but
    // reading the session directly for targeted checks.
    //
    // Here we use the public fn entry-points + a parallel CDP probe
    // (reusing the session file) to verify the bridge is live and
    // the serialized trees have the shape we expect.
    let page = browser_cli::attach(&demo)
        .await
        .expect("reattach to session");

    // Debug bridge live.
    let bridge_type = page.evaluate("typeof window.__jet_debug").await.unwrap();
    assert_eq!(
        bridge_type.as_str(),
        Some("object"),
        "window.__jet_debug should be an object"
    );

    // elementTree contains the <button>.
    let elt = page
        .evaluate("window.__jet_debug.elementTree()")
        .await
        .unwrap();
    let elt_str = serde_json::to_string(&elt).unwrap();
    assert!(
        elt_str.contains("\"button\""),
        "elementTree should contain button; got {elt_str}"
    );
    assert!(
        elt_str.contains("\"inc\""),
        "elementTree should contain id=inc; got {elt_str}"
    );

    // layoutTree has at least one node.
    let lt = page
        .evaluate("window.__jet_debug.layoutTree()")
        .await
        .unwrap();
    let lt_nodes = lt
        .get("nodes")
        .and_then(|v| v.as_array())
        .expect("layoutTree.nodes is an array");
    assert!(!lt_nodes.is_empty(), "layoutTree.nodes non-empty");

    // fiberTree has the Counter fiber with hook_count=1.
    let ft = page
        .evaluate("window.__jet_debug.fiberTree()")
        .await
        .unwrap();
    let fibers = ft.as_array().expect("fiberTree is an array");
    assert_eq!(fibers.len(), 1, "one fiber for Counter");
    assert_eq!(
        fibers[0].get("hook_count").and_then(|v| v.as_u64()),
        Some(1),
        "Counter has one useState"
    );

    // hookValues on fiber 0 returns the initial count as i64.
    let hv = page
        .evaluate("window.__jet_debug.hookValues(0)")
        .await
        .unwrap();
    let hv_arr = hv.as_array().expect("hookValues is an array");
    assert_eq!(hv_arr.len(), 1);
    assert_eq!(
        hv_arr[0].get("kind").and_then(|v| v.as_str()),
        Some("State")
    );
    assert_eq!(
        hv_arr[0].get("type_name").and_then(|v| v.as_str()),
        Some("i64"),
    );
    assert_eq!(hv_arr[0].get("value").and_then(|v| v.as_i64()), Some(0),);

    // pickAt finds the button at (30, 12) — counter-demo's button
    // sits at the top-left, height 24.
    let pick = page
        .evaluate("window.__jet_debug.pickAt(30, 12)")
        .await
        .unwrap();
    assert!(!pick.is_null(), "pickAt should return a node at (30,12)");
    let pick_idx = pick
        .get("index")
        .and_then(|v| v.as_u64())
        .expect("pick result has index");

    // paintOps has non-empty content after the first paint.
    let po = page
        .evaluate("window.__jet_debug.paintOps()")
        .await
        .unwrap();
    let po_arr = po
        .as_array()
        .expect("paintOps is an array after first frame");
    assert!(!po_arr.is_empty(), "paintOps has at least one op");

    // highlight + force_rerender. After highlight, paintOps should
    // include a StrokeRect with red color (0xff,0x33,0x33).
    let _ = page
        .evaluate(&format!("window.__jet_debug.highlight({})", pick_idx))
        .await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let po_hl = page
        .evaluate("window.__jet_debug.paintOps()")
        .await
        .unwrap();
    let po_hl_str = serde_json::to_string(&po_hl).unwrap();
    assert!(
        po_hl_str.contains("\"stroke_rect\""),
        "highlight should emit a StrokeRect op; got {po_hl_str}"
    );
    assert!(
        po_hl_str.contains("\"r\":255")
            && po_hl_str.contains("\"g\":51")
            && po_hl_str.contains("\"b\":51"),
        "highlight color should be red (255,51,51); got {po_hl_str}"
    );

    // Drive a real CDP mouse sequence through the Jet browser command path to
    // increment n, then verify hookValues reflects it.
    let canvas_rect = page
        .evaluate(
            "(() => {\
               const r = document.getElementById('jet-canvas').getBoundingClientRect();\
               return { left: r.left, top: r.top };\
             })()",
        )
        .await
        .unwrap();
    let click_x = canvas_rect
        .get("left")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
        + 30.0;
    let click_y = canvas_rect
        .get("top")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
        + 12.0;
    browser_cli::drag(&demo, click_x, click_y, click_x, click_y, 1)
        .await
        .expect("jet browser drag should click the counter button");
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let hv2 = page
        .evaluate("window.__jet_debug.hookValues(0)")
        .await
        .unwrap();
    assert_eq!(
        hv2.as_array()
            .and_then(|a| a.first())
            .and_then(|h| h.get("value"))
            .and_then(|v| v.as_i64()),
        Some(1),
        "hook value should be 1 after click",
    );

    let bundle = browser_cli::observation_bundle(&demo, &[])
        .await
        .expect("capture observation bundle");
    assert_eq!(
        bundle.get("schema_version").and_then(|v| v.as_str()),
        Some("jet.browser.observation.v1")
    );
    let build_artifact = bundle
        .get("build_artifact")
        .expect("bundle.build_artifact is present");
    assert_eq!(
        build_artifact.get("present").and_then(|v| v.as_bool()),
        Some(true),
        "browser capture must include dist/jet-target.json build evidence: {build_artifact:?}"
    );
    let target_manifest = build_artifact
        .get("manifest")
        .expect("bundle.build_artifact.manifest is present");
    assert_eq!(
        target_manifest
            .pointer("/build/mode")
            .and_then(|v| v.as_str()),
        Some("dev"),
        "browser capture must prove the debug/dev WASM build mode: {target_manifest:?}"
    );
    assert_eq!(
        target_manifest
            .pointer("/build/tsx_lowering")
            .and_then(|v| v.as_str()),
        Some(wasm_manifest::TSX_LOWERING_STRICT),
        "browser capture must prove strict TSX lowering: {target_manifest:?}"
    );
    let screenshot_visual_probe = bundle
        .get("screenshot_visual_probe")
        .expect("bundle.screenshot_visual_probe is present");
    assert!(
        screenshot_visual_probe.get("error").is_none(),
        "browser capture screenshot visual probe must not fail: {screenshot_visual_probe:?}"
    );
    assert_eq!(
        screenshot_visual_probe
            .get("schema_version")
            .and_then(|v| v.as_str()),
        Some("jet.browser.screenshot_visual_probe.v1"),
        "browser capture screenshot probe schema must be stable: {screenshot_visual_probe:?}"
    );
    assert!(
        screenshot_visual_probe
            .get("hash")
            .and_then(|v| v.as_str())
            .is_some_and(|hash| hash.len() == 16 && hash.chars().all(|ch| ch.is_ascii_hexdigit()))
            && screenshot_visual_probe
                .get("hashOnes")
                .and_then(|v| v.as_u64())
                .is_some_and(|ones| ones <= 64),
        "browser capture screenshot probe must include a comparable 64-bit visual hash: {screenshot_visual_probe:?}"
    );
    assert!(
        screenshot_visual_probe
            .get("pngByteLen")
            .and_then(|v| v.as_u64())
            .is_some_and(|count| count > 1_000)
            && screenshot_visual_probe
                .get("nonTransparent")
                .and_then(|v| v.as_u64())
                .is_some_and(|count| count > 0)
            && screenshot_visual_probe
                .get("nonWhite")
                .and_then(|v| v.as_u64())
                .is_some_and(|count| count > 0)
            && screenshot_visual_probe
                .get("nonBlack")
                .and_then(|v| v.as_u64())
                .is_some_and(|count| count > 0)
            && screenshot_visual_probe
                .get("uniqueBuckets")
                .and_then(|v| v.as_u64())
                .is_some_and(|count| count >= 2),
        "browser capture must include nonblank screenshot pixel evidence: {screenshot_visual_probe:?}"
    );
    let canvas_visual_probe = bundle
        .get("runtime")
        .and_then(|v| v.get("canvas_visual_probe"))
        .expect("bundle.runtime.canvas_visual_probe is present");
    assert!(
        canvas_visual_probe.get("error").is_none(),
        "browser capture canvas visual probe must not fail: {canvas_visual_probe:?}"
    );
    let webgpu_status = bundle
        .get("runtime")
        .and_then(|v| v.get("webgpu_status"))
        .expect("bundle.runtime.webgpu_status is present");
    assert_eq!(
        webgpu_status.get("bridgeMode").and_then(|v| v.as_str()),
        Some("text"),
        "browser capture must include the live WebGPU text bridge status: {webgpu_status:?}"
    );
    assert_eq!(
        webgpu_status
            .get("textAtlasMode")
            .and_then(|v| v.as_str()),
        Some("glyph-atlas"),
        "browser capture must prove the text pass is sampling a real glyph atlas: {webgpu_status:?}"
    );
    assert!(
        webgpu_status
            .get("lastTextGlyphCount")
            .and_then(|v| v.as_u64())
            .is_some_and(|count| count > 0),
        "browser capture must include a non-empty text glyph count: {webgpu_status:?}"
    );
    assert!(
        webgpu_status
            .get("lastTextAtlasUploadCount")
            .and_then(|v| v.as_u64())
            .is_some_and(|count| count >= 1),
        "browser capture must include a real atlas upload count: {webgpu_status:?}"
    );
    assert!(
        webgpu_status
            .get("lastTextAtlasNonZeroAlphaCount")
            .and_then(|v| v.as_u64())
            .is_some_and(|count| count > 0),
        "browser capture must include non-zero glyph alpha evidence: {webgpu_status:?}"
    );
    assert_gpu_timing_status_observable(webgpu_status);
    let bundle_layout_nodes = bundle
        .get("layout_tree")
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array())
        .expect("bundle.layout_tree.nodes is an array");
    assert!(
        !bundle_layout_nodes.is_empty(),
        "bundle layout evidence should be non-empty"
    );
    let bundle_paint_ops = bundle
        .get("paint_ops")
        .and_then(|v| v.as_array())
        .expect("bundle.paint_ops is an array");
    assert!(
        !bundle_paint_ops.is_empty(),
        "bundle paint evidence should be non-empty"
    );
    let bundle_hook_value = bundle
        .get("hook_values")
        .and_then(|v| v.as_array())
        .and_then(|items| {
            items
                .iter()
                .find(|item| item.get("fiber_id").and_then(|v| v.as_u64()) == Some(0))
        })
        .and_then(|item| item.get("values"))
        .and_then(|values| values.as_array())
        .and_then(|values| values.first())
        .and_then(|hook| hook.get("value"))
        .and_then(|value| value.as_i64());
    assert_eq!(
        bundle_hook_value,
        Some(1),
        "bundle should include post-click counter hook value"
    );

    // Clean up: close Chromium + stop the dev server + clear session.
    browser_cli::session::clear(&demo);
    let _ = browser.close().await;
    serve.abort();
}
// CODEGEN-END
