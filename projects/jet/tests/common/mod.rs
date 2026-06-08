// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
// CODEGEN-BEGIN
//! Shared harness for jet-wasm integration tests.
//!
//! Tiered to match `conformance.md`:
//!
//! - **Launch + click + shutdown** — framework-agnostic. Any
//!   adapter that produces `jet_wasm::Element` trees works here.
//! - **`element_tree` / `layout_tree` / `paint_ops` / `pick_at`** —
//!   framework-agnostic observable oracle. Every adapter's test
//!   suite is expected to lean on these.
//! - **`fiber_tree` / `hook_values`** — React-specific. Marked in
//!   their doc comments; tests for future Vue / Angular / Solid
//!   adapters must not call these.
//!
//! Snapshot-based assertions live in `snapshot.rs`. Prefer them
//! over hand-written equality for tree-shape checks — drift shows
//! up as a JSON diff instead of a one-off `assert_eq!` failure.
//!
//! Not part of the published `jet` API — stays test-only until the
//! shape stabilises across 20+ tests. See `conformance.md`
//! §Process — adding a new feature.

#![allow(dead_code)] // each test only uses some of the helpers.

pub mod canvas_spy;
pub mod react_oracle;
pub mod snapshot;

use anyhow::{anyhow, ensure, Result};
use jet::browser::{page::Page, Browser};
use jet::browser_cli;
use jet::wasm_build::{self, Profile};
use jet::wasm_dev::{self, DevOptions};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const WASM_E2E_READY_ATTEMPTS: usize = 180;
pub const WASM_E2E_READY_INTERVAL: Duration = Duration::from_millis(500);

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn node_available() -> bool {
    which::which("node").is_ok()
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn python_available() -> bool {
    which::which("python3").is_ok()
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn wasm_pack_available() -> bool {
    which::which("wasm-pack").is_ok()
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn chromium_available() -> bool {
    if std::env::var("CHROME_PATH").is_ok() {
        return true;
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let xdg = std::env::var("XDG_CACHE_HOME").unwrap_or_else(|_| format!("{home}/.cache"));
    [
        format!("{home}/Library/Caches/ms-playwright"),
        format!("{xdg}/ms-playwright"),
        format!("{home}/.jet/browsers"),
    ]
    .iter()
    .any(|p| Path::new(p).exists())
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn fail_missing_prerequisite(message: String) -> ! {
    panic!("{message}");
}

/// Require wasm-pack and Chromium for browser-backed WASM E2E tests.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn require_env() {
    if !wasm_pack_available() || !chromium_available() {
        fail_missing_prerequisite(format!(
            "need wasm-pack + Chromium (wasm-pack={}, chromium={})",
            wasm_pack_available(),
            chromium_available(),
        ));
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn require_wasm_pack_env() {
    if !wasm_pack_available() {
        fail_missing_prerequisite("need wasm-pack on PATH".to_string());
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn require_full_wasm_e2e_env() {
    if !node_available() || !python_available() || !wasm_pack_available() || !chromium_available() {
        fail_missing_prerequisite(format!(
            "need node + python3 + wasm-pack + Chromium \
             (node={} python={} wasm-pack={} chromium={})",
            node_available(),
            python_available(),
            wasm_pack_available(),
            chromium_available(),
        ));
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub async fn wait_for_http_ready(client: &reqwest::Client, url: &str) -> bool {
    for _ in 0..WASM_E2E_READY_ATTEMPTS {
        if client.get(url).send().await.is_ok() {
            return true;
        }
        tokio::time::sleep(WASM_E2E_READY_INTERVAL).await;
    }
    false
}

async fn free_port() -> u16 {
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    l.local_addr().unwrap().port()
}

/// Everything a test needs to talk to a live jet-wasm app.
///
/// Drop (or explicit `shutdown().await`) closes Chromium + stops the
/// dev server + clears the session file.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub struct JetTestApp {
    pub demo_dir: PathBuf,
    pub page: Page,
    pub url: String,
    browser: Browser,
    serve_task: tokio::task::JoinHandle<Result<()>>,
}

// ── Framework-agnostic: launch + shutdown + click ───────────────────────────

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
impl JetTestApp {
    /// `example_name` is a dir under the workspace's `examples/`.
    /// Must contain a `jet.config.toml` with a `[wasm]` section +
    /// the matching entry file (`.tsx` today; `.vue` / `.ng.ts` in
    /// future adapter branches).
    pub async fn launch(example_name: &str) -> Result<Self> {
        Self::launch_with_init_scripts(example_name, &[]).await
    }

    /// Launch a Jet WASM debug app with browser init scripts installed before
    /// `boot.js` runs.
    pub async fn launch_with_init_scripts(
        example_name: &str,
        init_scripts: &[&str],
    ) -> Result<Self> {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crates/")
            .parent()
            .expect("workspace root")
            .to_path_buf();
        let demo = workspace.join("examples").join(example_name);
        Self::launch_project_with_init_scripts(&demo, init_scripts).await
    }

    /// Launch a Jet WASM debug app from an explicit project root.
    /// @spec .aw/tech-design/projects/jet/specs/3943.md#changes
    pub async fn launch_project(root_dir: &Path) -> Result<Self> {
        Self::launch_project_with_init_scripts(root_dir, &[]).await
    }

    /// Launch a Jet WASM debug app from an explicit project root with browser
    /// init scripts installed before `boot.js` runs.
    /// @spec .aw/tech-design/projects/jet/specs/3943.md#changes
    pub async fn launch_project_with_init_scripts(
        root_dir: &Path,
        init_scripts: &[&str],
    ) -> Result<Self> {
        let demo = root_dir.to_path_buf();
        ensure!(demo.exists(), "missing example dir {}", demo.display());

        // Clean prior dist so each run exercises a fresh build path.
        // Keep .jet/wasm-build/ — wasm-pack incremental saves ~30s
        // per rerun and we've never seen staleness bite us.
        let _ = std::fs::remove_dir_all(demo.join("dist"));

        wasm_build::build_with_profile(
            &demo,
            Path::new("dist"),
            Profile::Dev,
            jet::build_target::BuildTarget::Web,
        )?;

        let mut last_err = None;
        for attempt in 0..2 {
            let port = free_port().await;
            let url = format!("http://127.0.0.1:{port}/");

            let serve_root = demo.clone();
            let serve_task = tokio::spawn(async move {
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

            // Poll for readiness. Cold wasm-pack + wasm-bindgen can take
            // tens of seconds on fresh caches; keep this above the
            // release smoke budget so E2E does not fail just before bind.
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(2))
                .build()?;
            if !wait_for_http_ready(&client, &url).await {
                serve_task.abort();
                last_err = Some(anyhow!("wasm_dev never came up at {url}"));
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }

            match browser_cli::prepare_session_with_init_scripts(&demo, &url, init_scripts).await {
                Ok(browser) => {
                    // wasm-bindgen init + first paint + JetDebug registration.
                    tokio::time::sleep(Duration::from_millis(1500)).await;
                    match browser_cli::attach(&demo).await {
                        Ok(page) => {
                            return Ok(Self {
                                demo_dir: demo,
                                page,
                                url,
                                browser,
                                serve_task,
                            });
                        }
                        Err(err) => {
                            let _ = browser.close().await;
                            serve_task.abort();
                            browser_cli::session::clear(&demo);
                            last_err = Some(err);
                        }
                    }
                }
                Err(err) => {
                    serve_task.abort();
                    browser_cli::session::clear(&demo);
                    last_err = Some(err);
                }
            }

            if attempt == 0 {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        Err(last_err.unwrap_or_else(|| anyhow!("Jet test app launch failed without error")))
    }

    /// Click the canvas at (x, y) via a synthetic DOM MouseEvent.
    /// Using DOM dispatch rather than CDP Input makes timing
    /// deterministic — no wait for OS-level event plumbing.
    pub async fn click_canvas(&self, x: f32, y: f32) -> Result<()> {
        let expr = format!(
            "(() => {{\
               const c = document.getElementById('jet-canvas');\
               const r = c.getBoundingClientRect();\
               c.dispatchEvent(new MouseEvent('click', {{\
                 clientX: r.left + {x}, clientY: r.top + {y},\
                 bubbles: true, cancelable: true, view: window\
               }}));\
             }})()"
        );
        self.page.evaluate(&expr).await?;
        tokio::time::sleep(Duration::from_millis(150)).await;
        Ok(())
    }

    /// Tear down the browser + dev server. Safe to call multiple
    /// times; a no-op if already shut down.
    pub async fn shutdown(self) {
        browser_cli::session::clear(&self.demo_dir);
        let _ = self.browser.close().await;
        self.serve_task.abort();
    }
}

// ── Framework-agnostic observable oracle ────────────────────────────────────
//
// Any TSX / Vue / Angular adapter that targets `jet_wasm::Element`
// supports these. Test bodies that restrict themselves to this
// impl block port straight to future adapters.

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
impl JetTestApp {
    /// Serialized snapshot of the currently-mounted Element tree.
    /// Primary observable — most assertions go through here.
    pub async fn element_tree(&self) -> Result<serde_json::Value> {
        Ok(self
            .page
            .evaluate("window.__jet_debug.elementTree()")
            .await?)
    }

    /// Serialized last-laid-out tree (rects + tags).
    pub async fn layout_tree(&self) -> Result<serde_json::Value> {
        Ok(self
            .page
            .evaluate("window.__jet_debug.layoutTree()")
            .await?)
    }

    /// Last-frame PaintOps. `null` before first paint.
    pub async fn paint_ops(&self) -> Result<serde_json::Value> {
        Ok(self.page.evaluate("window.__jet_debug.paintOps()").await?)
    }

    /// Hit-test a canvas-space point. Returns `{index, node}` or null.
    pub async fn pick_at(&self, x: f32, y: f32) -> Result<serde_json::Value> {
        Ok(self
            .page
            .evaluate(&format!("window.__jet_debug.pickAt({x}, {y})"))
            .await?)
    }
}

// ── React-specific surface ──────────────────────────────────────────────────
//
// These methods assume the app was built against `jet_wasm::react`.
// Vue / Angular / Solid adapters expose different inspection
// surfaces and tests for those adapters must NOT call these —
// they'd come back empty or nonsensical.
//
// Kept in a separate impl block as a readability hint: whenever a
// test file reaches into this section, it's making a React-specific
// assertion and cannot port as-is to another adapter.

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
impl JetTestApp {
    /// **React-only.** Flat list of fibers with hook_count + dirty.
    pub async fn fiber_tree(&self) -> Result<serde_json::Value> {
        Ok(self.page.evaluate("window.__jet_debug.fiberTree()").await?)
    }

    /// **React-only.** Per-slot summary of the given fiber's hooks.
    pub async fn hook_values(&self, fiber_id: u32) -> Result<serde_json::Value> {
        Ok(self
            .page
            .evaluate(&format!("window.__jet_debug.hookValues({fiber_id})"))
            .await?)
    }
}
// CODEGEN-END
