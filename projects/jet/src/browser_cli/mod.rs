// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
// CODEGEN-BEGIN
//! `jet browser <subcommand>` — interactive debugging CLI that
//! attaches to a live jet-wasm app built with `--debug`.
//!
//! Flow:
//!
//! 1. `jet browser launch <url>` boots Chromium via `browser::Browser`,
//!    navigates to `<url>`, writes `.jet/browser-session.json`, and
//!    blocks until Ctrl-C. The browser stays alive while this
//!    command is running.
//!
//! 2. From a second terminal, every other `jet browser *` command
//!    reads the session file, reattaches to the same target, and
//!    drives it via `Page::evaluate("window.__jet_debug.<method>()")`.
//!
//! The CLI talks to jet-wasm's runtime purely through the `JetDebug`
//! bridge on `window.__jet_debug`. Non-debug builds expose nothing
//! there; we detect that and print a clear hint.

pub mod pretty;
pub mod session;

use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::env::VarError;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::browser::{cdp::CdpClient, page::Page};

/// GH #3612 — distinguish `Err(VarError::NotPresent)` (canonical "no
/// override", silent fall-through to autodiscovery) from
/// `Err(VarError::NotUnicode(_))` (real misconfiguration: the user
/// set CHROME_PATH but jet silently discards their override).
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub(crate) fn safe_chrome_path_override(
    current: Result<String, VarError>,
) -> (Option<PathBuf>, Option<String>) {
    match current {
        Ok(p) => (Some(PathBuf::from(p)), None),
        Err(VarError::NotPresent) => (None, None),
        Err(VarError::NotUnicode(_)) => (None, Some(format_safe_chrome_path_warn("not-unicode"))),
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub(crate) fn format_safe_chrome_path_warn(observed_kind: &str) -> String {
    format!(
        "GH #3612 browser_cli: CHROME_PATH observed as {observed_kind}; \
         your explicit Chromium binary override is being SILENTLY DROPPED \
         — jet will fall through to launcher autodiscovery, which may pick \
         a different binary or fail. Re-set CHROME_PATH with a valid UTF-8 \
         path."
    )
}

/// Connect to the session file's Chromium and attach to its target.
/// Common prelude for every read-only command. `pub` so integration
/// tests can exercise the same session without re-implementing the
/// connect dance.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub async fn attach(root_dir: &Path) -> Result<Page> {
    let s = session::read_live(root_dir)?;
    let client = CdpClient::connect(&s.ws_endpoint).await.with_context(|| {
        format!(
            "connecting to {} — browser likely closed; run `jet browser launch <url>` again",
            s.ws_endpoint
        )
    })?;
    let sess = client
        .attach_to_target(&s.target_id)
        .await
        .with_context(|| format!("attaching to target {}", s.target_id))?;
    Ok(Page::new(sess, s.target_id))
}

/// Probe whether `window.__jet_debug` exists on the page. We use
/// this before every command so the user gets a useful error
/// instead of a confusing "undefined" result.
async fn assert_debug_bridge(page: &Page) -> Result<()> {
    let v = page
        .evaluate("typeof window.__jet_debug")
        .await
        .context("probing window.__jet_debug")?;
    if v.as_str() == Some("undefined") {
        bail!(
            "app is not built with --debug (window.__jet_debug is undefined). \
             Rebuild with `jet build --wasm --debug` or run `jet dev --wasm --debug`."
        );
    }
    Ok(())
}

/// Canonical expression wrapping. We ask CDP for JSON via the return
/// value directly — the JetDebug methods already return
/// JsValue-serialized JSON objects.
fn expr(method: &str, args: &str) -> String {
    format!("window.__jet_debug.{method}({args})")
}

// ── Subcommand entry points ─────────────────────────────────────────────────

/// Open Chromium, navigate to `url`, write the session file, and
/// return the live `Browser` so the caller can decide when to close
/// it. The CLI `launch` / `debug` entry points block on Ctrl-C;
/// integration tests can drive the returned handle directly.
///
/// Respects `CHROME_PATH` env var as an executable override so CI
/// and integration tests can target Playwright's headless Chromium
/// even when a system Google Chrome is also present.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub async fn prepare_session(root_dir: &Path, url: &str) -> Result<crate::browser::Browser> {
    prepare_session_with_init_scripts(root_dir, url, &[]).await
}

/// Like [`prepare_session`], but registers init scripts before navigation.
///
/// Tests use this to install observation hooks such as the canvas 2D spy
/// before `boot.js` imports and starts the WASM application.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub async fn prepare_session_with_init_scripts(
    root_dir: &Path,
    url: &str,
    init_scripts: &[&str],
) -> Result<crate::browser::Browser> {
    use crate::browser::{Browser, LaunchOptions};
    eprintln!("[jet browser] launching Chromium…");
    let mut options = LaunchOptions::default();
    // GH #3612 — distinguish CHROME_PATH NotPresent (silent) from
    // NotUnicode (warn). The prior `if let Ok(p) = ...` silently
    // dropped a misconfigured CHROME_PATH on the floor.
    let (chrome_override, chrome_warn) = safe_chrome_path_override(std::env::var("CHROME_PATH"));
    if let Some(msg) = chrome_warn {
        tracing::warn!(target: "jet::browser_cli", "{}", msg);
    }
    if let Some(p) = chrome_override {
        options.executable = Some(p);
    }
    let browser = Browser::launch(options)
        .await
        .context("launching Chromium")?;
    let page = browser.new_page().await.context("opening new page")?;
    for source in init_scripts {
        page.add_init_script(source)
            .await
            .context("registering browser init script")?;
    }
    page.goto(url)
        .await
        .with_context(|| format!("navigating to {url}"))?;

    let s = session::Session {
        ws_endpoint: browser.ws_url().to_string(),
        target_id: page.target_id().to_string(),
        url: url.to_string(),
        pid: std::process::id(),
        started_at: session::now_unix(),
    };
    session::write(root_dir, &s)?;
    Ok(browser)
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub async fn launch(root_dir: &Path, url: &str) -> Result<()> {
    session::clear_shutdown_request(root_dir);
    let browser = prepare_session(root_dir, url).await?;
    eprintln!(
        "[jet browser] session ready. In another terminal try:\n    \
         jet browser tree\n    \
         jet browser pick\n    \
         jet browser hooks 0\n\
         Ctrl-C to shut the browser down."
    );

    // GH #3732 — was `let _ = tokio::signal::ctrl_c().await;` which
    // silently swallowed handler-registration errors. When ctrl_c
    // returns `Err` (signal limits exhausted, sandboxed runtime
    // forbidding `sigaction`, etc.), the await returned immediately,
    // `let _ =` dropped the error, and the function proceeded to print
    // "shutting down", clear the session file, and close the browser
    // — all without any Ctrl+C from the user. Worse than #3725/#3730
    // because `session::clear(root_dir)` writes to disk, so the side
    // effect persisted across the spurious exit, leaving sibling
    // `jet browser tree/pick/hooks` commands without a session.
    // Match on the result: on Ok run the documented cleanup; on Err
    // warn and park forever so the browser stays attached.
    let ctrl_c = async {
        match tokio::signal::ctrl_c().await {
            Ok(()) => "Ctrl-C",
            Err(err) => {
                tracing::warn!(
                    target: "jet::browser_cli",
                    error = %err,
                    "{}",
                    format_browser_cli_ctrl_c_warn(&err)
                );
                std::future::pending::<&'static str>().await
            }
        }
    };
    let shutdown_request = wait_for_shutdown_request(root_dir);
    let reason = tokio::select! {
        reason = ctrl_c => reason,
        _ = shutdown_request => "jet browser shutdown",
    };
    eprintln!("[jet browser] shutting down ({reason}).");
    session::clear_shutdown_request(root_dir);
    session::clear(root_dir);
    let _ = browser.close().await;
    Ok(())
}

async fn wait_for_shutdown_request(root_dir: &Path) {
    loop {
        if session::shutdown_requested(root_dir) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub async fn shutdown(root_dir: &Path) -> Result<()> {
    let existing = session::read(root_dir).ok();
    session::request_shutdown(root_dir)?;

    if let Some(s) = existing {
        eprintln!("[jet browser] shutdown requested for launch pid {}", s.pid);
    } else {
        eprintln!("[jet browser] shutdown requested; no current session file was readable.");
    }
    Ok(())
}

/// GH #3732 — build the warn wording for the browser_cli ctrl_c handler
/// registration failure branch. Extracted so the issue tag, observable
/// symptoms (immediate exit + session file would be wiped), operator
/// stop-the-server guidance, and sibling cross-references are
/// unit-testable without provoking the actual signal-registration-
/// failure platform case. Sibling of `dev_server::format_dev_server_ctrl_c_warn`
/// (#3725) and `wasm_dev::format_wasm_dev_ctrl_c_warn` (#3730).
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub(crate) fn format_browser_cli_ctrl_c_warn(err: &std::io::Error) -> String {
    format!(
        "GH #3732 jet::browser_cli: failed to install the Ctrl+C handler \
         ({err}). Previously this site called `let _ =` which swallowed \
         the error — the await then returned immediately and `jet browser \
         launch` proceeded to print \"shutting down\", clear the session \
         file, and close the browser, all without any Ctrl+C from the \
         user. The disk-side effect (session::clear) persisted across \
         the spurious exit, so sibling `jet browser tree/pick/hooks` \
         commands lost their session. The browser will keep running; \
         stop it by sending SIGTERM (e.g. `kill <pid>`) or SIGKILL from \
         another terminal — the session file is preserved so siblings \
         can still attach. Fix the underlying cause by checking signal \
         limits (`ulimit -i`) and that your runtime permits `sigaction`. \
         Sibling of GH #3725 (dev_server) and GH #3730 (wasm_dev)."
    )
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub async fn tree(root_dir: &Path, which: &str) -> Result<()> {
    let page = attach(root_dir).await?;
    assert_debug_bridge(&page).await?;
    let (method, printer): (&str, fn(&Value) -> String) = match which {
        "element" => ("elementTree", pretty::element_tree),
        "layout" => ("layoutTree", pretty::layout_tree),
        "fiber" => ("fiberTree", pretty::fiber_tree),
        other => bail!("unknown tree kind {other:?} — use element | layout | fiber"),
    };
    let v = page.evaluate(&expr(method, "")).await?;
    print!("{}", printer(&v));
    Ok(())
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub async fn hooks(root_dir: &Path, fiber_id: u64) -> Result<()> {
    let page = attach(root_dir).await?;
    assert_debug_bridge(&page).await?;
    let v = page
        .evaluate(&expr("hookValues", &fiber_id.to_string()))
        .await?;
    print!("{}", pretty::hook_values(&v));
    Ok(())
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub async fn pick(root_dir: &Path, timeout_secs: u64) -> Result<()> {
    let page = attach(root_dir).await?;
    assert_debug_bridge(&page).await?;

    // Arm a one-shot listener that stashes the hit-tested node on
    // `window.__jet_debug_pick`, then poll from Rust until the
    // global appears. A raw eval is simpler than wiring a new
    // PageRequest variant + it fits on one screen.
    //
    // The listener must use { once: true, capture: true } so it
    // fires before the app's own `click` listener — otherwise
    // setting state before we read coords races the repaint.
    let arm = r#"
        (() => {
            delete window.__jet_debug_pick;
            const c = document.getElementById('jet-canvas');
            if (!c) return 'no canvas';
            const fn = (e) => {
                const r = c.getBoundingClientRect();
                const x = e.clientX - r.left;
                const y = e.clientY - r.top;
                window.__jet_debug_pick = window.__jet_debug.pickAt(x, y) || { index: null };
            };
            c.addEventListener('click', fn, { once: true, capture: true });
            return 'armed';
        })()
    "#;
    let armed = page.evaluate(arm).await?;
    if armed.as_str() != Some("armed") {
        bail!("could not arm pick listener: {armed}");
    }
    eprintln!("[jet browser pick] click on the canvas in Chromium (timeout {timeout_secs}s)…");

    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        if std::time::Instant::now() >= deadline {
            bail!("pick timed out after {timeout_secs}s");
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
        let v = page.evaluate("window.__jet_debug_pick ?? null").await?;
        if !v.is_null() {
            println!("{}", serde_json::to_string_pretty(&v)?);
            return Ok(());
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub async fn highlight(root_dir: &Path, index: Option<usize>) -> Result<()> {
    let page = attach(root_dir).await?;
    assert_debug_bridge(&page).await?;
    let arg = match index {
        Some(i) => i.to_string(),
        None => "undefined".to_string(),
    };
    page.evaluate(&expr("highlight", &arg)).await?;
    Ok(())
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub async fn frame(root_dir: &Path) -> Result<()> {
    let page = attach(root_dir).await?;
    assert_debug_bridge(&page).await?;
    let v = page.evaluate(&expr("paintOps", "")).await?;
    print!("{}", pretty::paint_ops(&v));
    Ok(())
}

/// Capture one machine-readable observation bundle from the attached
/// jet-wasm debug session. This is intentionally raw JSON: parity
/// tooling needs stable evidence, not the human pretty-printers used
/// by `tree`, `hooks`, and `frame`.
/// @spec .aw/tech-design/projects/jet/logic/jet-browser-observation-bundle.md#changes
pub async fn observation_bundle(root_dir: &Path, requested_hook_ids: &[u64]) -> Result<Value> {
    let page = attach(root_dir).await?;
    assert_debug_bridge(&page).await?;

    let runtime = page
        .evaluate(
            r#"
            (() => {
                const canvas = document.getElementById('jet-canvas');
                const rect = canvas ? canvas.getBoundingClientRect() : null;
                return {
                    url: window.location.href,
                    title: document.title || "",
                    viewport: {
                        width: window.innerWidth,
                        height: window.innerHeight,
                        device_pixel_ratio: window.devicePixelRatio || 1
                    },
                    canvas: canvas ? {
                        present: true,
                        id: canvas.id || "",
                        width: canvas.width || 0,
                        height: canvas.height || 0,
                        client_width: rect ? rect.width : 0,
                        client_height: rect ? rect.height : 0
                    } : {
                        present: false
                    }
                };
            })()
            "#,
        )
        .await
        .context("capturing runtime metadata")?;
    let element_tree = page
        .evaluate(&expr("elementTree", ""))
        .await
        .context("capturing element tree")?;
    let layout_tree = page
        .evaluate(&expr("layoutTree", ""))
        .await
        .context("capturing layout tree")?;
    let fiber_tree = page
        .evaluate(&expr("fiberTree", ""))
        .await
        .context("capturing fiber tree")?;
    let paint_ops = page
        .evaluate(&expr("paintOps", ""))
        .await
        .context("capturing paint ops")?;

    let hook_ids: Vec<u64> = if requested_hook_ids.is_empty() {
        fiber_tree
            .as_array()
            .into_iter()
            .flatten()
            .filter(|fiber| {
                fiber
                    .get("hook_count")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(0)
                    > 0
            })
            .filter_map(|fiber| fiber.get("id").and_then(|value| value.as_u64()))
            .collect()
    } else {
        requested_hook_ids.to_vec()
    };
    let mut hook_values = Vec::new();
    for fiber_id in hook_ids {
        let values = page
            .evaluate(&expr("hookValues", &fiber_id.to_string()))
            .await
            .with_context(|| format!("capturing hook values for fiber {fiber_id}"))?;
        hook_values.push(serde_json::json!({
            "fiber_id": fiber_id,
            "values": values,
        }));
    }

    Ok(serde_json::json!({
        "schema_version": "jet.browser.observation.v1",
        "runtime": runtime,
        "bridge": {
            "available": true,
            "methods": [
                "elementTree",
                "layoutTree",
                "fiberTree",
                "hookValues",
                "paintOps",
                "pickAt",
                "highlight",
                "forceRerender"
            ]
        },
        "element_tree": element_tree,
        "layout_tree": layout_tree,
        "fiber_tree": fiber_tree,
        "hook_values": hook_values,
        "paint_ops": paint_ops,
    }))
}

/// Capture one machine-readable observation bundle from a live DOM page.
/// The tree shape intentionally matches the React DOM oracle test normalizer
/// so browser-capture evidence can be compared directly against jet-wasm.
/// @spec .aw/tech-design/projects/jet/specs/3941.md#changes
pub async fn dom_observation_bundle_from_page(page: &Page, root_selector: &str) -> Result<Value> {
    let runtime = page
        .evaluate(
            r#"
            (() => ({
                url: window.location.href,
                title: document.title || "",
                viewport: {
                    width: window.innerWidth,
                    height: window.innerHeight,
                    device_pixel_ratio: window.devicePixelRatio || 1
                }
            }))()
            "#,
        )
        .await
        .context("capturing DOM runtime metadata")?;
    let dom_tree = page
        .evaluate(&dom_tree_expr(root_selector)?)
        .await
        .with_context(|| format!("capturing DOM tree for selector {root_selector:?}"))?;

    Ok(serde_json::json!({
        "schema_version": "jet.browser.dom_observation.v1",
        "runtime": runtime,
        "root_selector": root_selector,
        "dom_tree": dom_tree,
    }))
}

/// @spec .aw/tech-design/projects/jet/specs/3941.md#changes
pub async fn dom_observation_bundle(root_dir: &Path, root_selector: &str) -> Result<Value> {
    let page = attach(root_dir).await?;
    dom_observation_bundle_from_page(&page, root_selector).await
}

fn dom_tree_expr(root_selector: &str) -> Result<String> {
    let selector = serde_json::to_string(root_selector)?;
    Ok(format!(
        r#"
(() => {{
  const root = document.querySelector({selector});
  const stableAttrs = new Set(['id', 'class', 'style', 'role', 'aria-label', 'data-testid']);
  const normalizeText = (text) => text.replace(/\s+/g, ' ').trim();
  const mergeText = (children) => {{
    const out = [];
    for (const child of children) {{
      if (child.kind === 'text' && normalizeText(child.text) === '') continue;
      const prev = out[out.length - 1];
      if (prev && prev.kind === 'text' && child.kind === 'text') {{
        prev.text = `${{prev.text}}${{child.text}}`;
      }} else {{
        out.push(child);
      }}
    }}
    return out;
  }};
  const finalizeText = (node) => {{
    if (!node) return null;
    if (node.kind === 'text') return {{ kind: 'text', text: normalizeText(node.text || '') }};
    if (node.kind === 'element') {{
      return {{ ...node, children: node.children.map(finalizeText).filter(Boolean) }};
    }}
    return node;
  }};
  const walk = (node) => {{
    if (!node) return null;
    if (node.nodeType === Node.TEXT_NODE) {{
      return {{ kind: 'text', text: node.textContent || '' }};
    }}
    if (node.nodeType !== Node.ELEMENT_NODE) return null;
    const attrs = {{}};
    for (const attr of Array.from(node.attributes)) {{
      if (stableAttrs.has(attr.name) || attr.name.startsWith('data-') || attr.name.startsWith('aria-')) {{
        attrs[attr.name] = attr.value;
      }}
    }}
    const children = mergeText(Array.from(node.childNodes).map(walk).filter(Boolean));
    return {{
      kind: 'element',
      tag: node.tagName.toLowerCase(),
      attrs,
      children
    }};
  }};
  return finalizeText(walk(root));
}})()
"#
    ))
}

/// @spec .aw/tech-design/projects/jet/logic/jet-browser-observation-bundle.md#changes
/// @spec .aw/tech-design/projects/jet/specs/3941.md#changes
pub async fn capture(
    root_dir: &Path,
    surface: &str,
    root_selector: &str,
    hook_ids: &[u64],
    pretty: bool,
    out_path: Option<&Path>,
) -> Result<()> {
    let bundle = match surface {
        "wasm" => observation_bundle(root_dir, hook_ids).await?,
        "dom" => {
            if !hook_ids.is_empty() {
                bail!("--hook is only supported with --surface wasm");
            }
            dom_observation_bundle(root_dir, root_selector).await?
        }
        other => bail!("unknown browser capture surface {other:?}; expected wasm or dom"),
    };
    let body = if pretty {
        serde_json::to_string_pretty(&bundle)?
    } else {
        serde_json::to_string(&bundle)?
    };
    match out_path {
        Some(path) => {
            std::fs::write(path, body).with_context(|| format!("writing {}", path.display()))?;
            eprintln!(
                "[jet browser] wrote observation bundle to {}",
                path.display()
            );
        }
        None => {
            println!("{body}");
        }
    }
    Ok(())
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub async fn screenshot(root_dir: &Path, out_path: Option<&Path>) -> Result<()> {
    let page = attach(root_dir).await?;
    let bytes = page.screenshot().await.context("capturing screenshot")?;
    match out_path {
        Some(p) => {
            std::fs::write(p, &bytes).with_context(|| format!("writing {}", p.display()))?;
            eprintln!(
                "[jet browser] wrote {} bytes to {}",
                bytes.len(),
                p.display()
            );
        }
        None => {
            use std::io::Write;
            std::io::stdout().write_all(&bytes)?;
        }
    }
    Ok(())
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub async fn eval(root_dir: &Path, expression: &str) -> Result<()> {
    let page = attach(root_dir).await?;
    let v = page.evaluate(expression).await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

/// Read `dist/tsx-source-map.json` and print where each component
/// was declared in the original TSX. Works offline — no browser
/// session needed. Filter by component name if supplied.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn tsx(root_dir: &Path, filter: Option<&str>) -> Result<()> {
    let map_path = root_dir.join("dist").join("tsx-source-map.json");
    let body = std::fs::read_to_string(&map_path).with_context(|| {
        format!(
            "reading {} — run `jet build --wasm [--debug]` first",
            map_path.display()
        )
    })?;
    let map: Value =
        serde_json::from_str(&body).with_context(|| format!("parsing {}", map_path.display()))?;
    let source = map
        .get("source_file")
        .and_then(|v| v.as_str())
        .unwrap_or("<unknown>");
    let empty: Vec<Value> = Vec::new();
    let components = map
        .get("components")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty);

    if components.is_empty() {
        println!("(no components in map)");
        return Ok(());
    }

    for c in components {
        let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("?");
        if let Some(f) = filter {
            if !name.contains(f) {
                continue;
            }
        }
        let line = c.get("tsx_line").and_then(|v| v.as_u64()).unwrap_or(0);
        let col = c.get("tsx_col").and_then(|v| v.as_u64()).unwrap_or(0);
        if source.is_empty() {
            // Release build: no source_file annotation, but
            // positions were still recorded.
            println!("{name:<20} line {line}, col {col}");
        } else {
            println!("{name:<20} {source}:{line}:{col}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod gh3612_safe_chrome_path_tests {
    //! GH #3612 — CHROME_PATH must distinguish NotPresent (silent
    //! fall-through to autodiscovery) from NotUnicode (warn). The prior
    //! `if let Ok(p) = ...` silently discarded a misconfigured override.
    use super::*;

    #[test]
    fn ok_path_is_used_as_override() {
        let (path, warn) = safe_chrome_path_override(Ok("/opt/chrome/chrome".to_string()));
        assert_eq!(path.unwrap(), PathBuf::from("/opt/chrome/chrome"));
        assert!(warn.is_none());
    }

    #[test]
    fn not_present_silently_skips_override() {
        let (path, warn) = safe_chrome_path_override(Err(VarError::NotPresent));
        assert!(path.is_none());
        assert!(
            warn.is_none(),
            "NotPresent is canonical — must not emit warn"
        );
    }

    #[test]
    fn not_unicode_skips_override_and_warns() {
        let raw = std::ffi::OsString::from("ignored");
        let (path, warn) = safe_chrome_path_override(Err(VarError::NotUnicode(raw)));
        assert!(path.is_none(), "no usable path → no override");
        let msg = warn.expect("NotUnicode must emit warn");
        assert!(msg.contains("GH #3612"), "msg: {msg}");
        assert!(msg.contains("not-unicode"), "msg: {msg}");
        assert!(msg.contains("CHROME_PATH"), "msg: {msg}");
    }

    #[test]
    fn warn_helper_names_consequences() {
        let msg = format_safe_chrome_path_warn("not-unicode");
        assert!(msg.contains("GH #3612"), "msg: {msg}");
        assert!(
            msg.to_lowercase().contains("dropped") || msg.to_lowercase().contains("override"),
            "must name dropped-override consequence, got: {msg}"
        );
    }

    /// Distinguishability: the two error discriminants must produce
    /// distinguishable warn states.
    #[test]
    fn discriminants_distinguishable() {
        let raw = std::ffi::OsString::from("ignored");
        let np = safe_chrome_path_override(Err(VarError::NotPresent)).1;
        let nu = safe_chrome_path_override(Err(VarError::NotUnicode(raw))).1;
        assert!(np.is_none());
        assert!(nu.is_some());
    }
}

#[cfg(test)]
mod gh3732_browser_cli_ctrl_c_warn_tests {
    //! GH #3732 — `browser_cli::launch` did
    //! `let _ = tokio::signal::ctrl_c().await;`, silently swallowing
    //! handler-registration errors. When ctrl_c returns `Err`, the
    //! await returned immediately and the function proceeded to
    //! "shutting down" — printing the shutdown message, calling
    //! `session::clear(root_dir)` (disk side effect!), and closing
    //! the browser, with no Ctrl+C from the user. Worse than #3725/
    //! #3730 because the session file got wiped, breaking sibling
    //! `jet browser tree/pick/hooks` commands.
    use super::*;

    fn sample_err() -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::PermissionDenied, "EPERM browser sample")
    }

    #[test]
    fn helper_tags_gh_issue() {
        let msg = format_browser_cli_ctrl_c_warn(&sample_err());
        assert!(msg.contains("GH #3732"), "must carry issue tag, got: {msg}");
    }

    #[test]
    fn helper_round_trips_io_error_text() {
        let msg = format_browser_cli_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("EPERM browser sample"),
            "must forward io::Error detail, got: {msg}"
        );
    }

    #[test]
    fn helper_names_immediate_exit_symptom() {
        let msg = format_browser_cli_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("immediately") || msg.contains("returned immediately"),
            "must explain immediate-exit symptom, got: {msg}"
        );
    }

    #[test]
    fn helper_calls_out_session_clear_disk_side_effect() {
        // This is the *worse* part vs #3725/#3730 — the spurious exit
        // also wiped the session file. The warn must name that so the
        // operator understands why sibling commands stopped working.
        let msg = format_browser_cli_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("session::clear")
                || msg.contains("session file")
                || msg.contains("session"),
            "must name session-file side effect, got: {msg}"
        );
        assert!(
            msg.contains("tree")
                || msg.contains("pick")
                || msg.contains("hooks")
                || msg.contains("sibling"),
            "must name the sibling commands that break, got: {msg}"
        );
    }

    #[test]
    fn helper_tells_user_how_to_stop_browser_without_wiping_session() {
        let msg = format_browser_cli_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("SIGTERM") || msg.contains("SIGKILL") || msg.contains("kill"),
            "must name how the operator stops the browser, got: {msg}"
        );
        assert!(
            msg.contains("preserved") || msg.contains("attach"),
            "must reassure that the session file survives the failure branch: {msg}"
        );
    }

    #[test]
    fn helper_points_at_signal_limits_root_cause() {
        let msg = format_browser_cli_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("signal") || msg.contains("sigaction") || msg.contains("ulimit"),
            "must point at signal-subsystem root cause, got: {msg}"
        );
    }

    #[test]
    fn helper_names_silent_fallback_root_cause_let_underscore_swallow() {
        let msg = format_browser_cli_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("let _ =") || msg.contains("swallowed"),
            "must call out the prior `let _ =` swallow, got: {msg}"
        );
    }

    #[test]
    fn helper_cross_references_both_sibling_issues() {
        let msg = format_browser_cli_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("GH #3725") || msg.contains("dev_server"),
            "must cross-reference dev_server sibling #3725: {msg}"
        );
        assert!(
            msg.contains("GH #3730") || msg.contains("wasm_dev"),
            "must cross-reference wasm_dev sibling #3730: {msg}"
        );
    }

    #[test]
    fn helper_is_deterministic_for_fixed_input() {
        let err = std::io::Error::new(std::io::ErrorKind::Other, "fixed-browser");
        let a = format_browser_cli_ctrl_c_warn(&err);
        let b = format_browser_cli_ctrl_c_warn(&err);
        assert_eq!(a, b);
    }

    #[test]
    fn helper_distinct_from_dev_server_3725_and_wasm_dev_3730_warns() {
        let me = format_browser_cli_ctrl_c_warn(&sample_err());
        let dev = crate::dev_server::format_dev_server_ctrl_c_warn(&sample_err());
        let wasm = crate::wasm_dev::format_wasm_dev_ctrl_c_warn(&sample_err());
        assert_ne!(me, dev);
        assert_ne!(me, wasm);
        // Sibling warns must not pose as us (no #3732 in their bodies
        // as the primary issue tag).
        assert!(
            !dev.contains("GH #3732"),
            "dev_server warn must not carry our tag: {dev}"
        );
        assert!(
            !wasm.contains("GH #3732"),
            "wasm_dev warn must not carry our tag: {wasm}"
        );
    }
}
// CODEGEN-END
