// HANDWRITE-BEGIN gap="missing-generator:unit-test:257c8ae8" tracker="pending-tracker" reason="Integration tests: `jet build --splitting`'s entry-flatten path (issue #1993) merges the safe subset of the entry chunk into one scope-hoisted region while keeping registry residue (eval/cycle/cross-chunk-referenced/scope-hoist-unsafe modules) on the pre-existing __jet__ registry; this file proves the combined build boots in a real browser (a lazily-loaded chunk still loads on demand, an entry-static shared chunk still resolves per #1963), that registry-residue code reading a flattened module's export gets the correct value via the __jet__.cache pre-seed interop shim (and the reverse direction: a flattened module's top-level code reading a registry-residue export), and that the flatten path measurably shrinks the entry file versus the JET_NO_ENTRY_FLATTEN=1 escape hatch."
//! `jet build --splitting` entry-flatten interop + size coverage (issue
//! #1993, child B of the beat-vite epic #1990).
//!
//! Harness (`run_jet`/`require_success`/`list_files_recursive`, the
//! `StaticDistServer` + `jet browser launch/eval/shutdown` real-Chromium
//! round-trip) is duplicated from `tests/build/code_splitting.rs` rather
//! than extracted into a shared helper, matching that file's own stated
//! precedent (its WI #1931 section header) that cross-file extraction here
//! would be its own out-of-scope refactor.

#[path = "../common/mod.rs"]
mod common;

use anyhow::{anyhow, Context, Result};
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde_json::Value;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::Duration;
use tokio::sync::oneshot;

fn run_jet<I, S>(fixture: &Path, args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_jet"))
        .args(args)
        .current_dir(fixture)
        .output()
        .context("run jet command")
}

fn require_success(output: Output, phase: &str) -> Result<Output> {
    if output.status.success() {
        return Ok(output);
    }
    Err(anyhow!(
        "{phase} failed\nstatus={}\nstdout={}\nstderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn list_files_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(list_files_recursive(&path));
        } else {
            out.push(path);
        }
    }
    out
}

fn find_entry_file(dist: &Path) -> PathBuf {
    list_files_recursive(dist)
        .into_iter()
        .find(|p| {
            p.parent() == Some(dist)
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("main.") && n.ends_with(".js"))
        })
        .expect("no top-level main.<hash>.js entry file")
}

/// Extracts the `flattened=N` (or `registry=M`) integer from a
/// `[bundle-timing] entry-flatten partition: flattened=N registry=M`
/// stderr line (`JET_BUNDLE_TIMING=1`; see
/// `Bundler::generate_split_bundle`, issue #1993).
fn parse_partition_count(stderr: &str, key: &str) -> Option<usize> {
    let line = stderr
        .lines()
        .find(|line| line.contains("entry-flatten partition:"))?;
    let needle = format!("{key}=");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

// ── real-Chromium interop smoke ─────────────────────────────────────────────
//
// Duplicated harness (see this file's module doc comment for why) from
// `tests/build/code_splitting.rs`'s #1931/#1963 real-browser smokes: serves
// a `--splitting` build's `dist/` over local HTTP and drives real Chromium
// through the `jet browser launch/eval/shutdown` CLI (a session file at
// `.jet/browser-session.json`, not an in-process CDP client).

struct StaticDistServer {
    url: String,
    shutdown: Option<oneshot::Sender<()>>,
}

impl StaticDistServer {
    async fn spawn(fixture: &Path) -> Result<Self> {
        let dist = fixture.join("dist");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .context("bind static dist server")?;
        let addr = listener
            .local_addr()
            .context("read static dist server addr")?;
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let app = Router::new()
            .fallback(get(serve_static_dist_request))
            .with_state(StaticDistState { dist });

        tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await;
        });

        Ok(Self {
            url: format!("http://{addr}/"),
            shutdown: Some(shutdown_tx),
        })
    }

    fn url(&self) -> String {
        self.url.clone()
    }
}

impl Drop for StaticDistServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

#[derive(Clone)]
struct StaticDistState {
    dist: PathBuf,
}

async fn serve_static_dist_request(State(state): State<StaticDistState>, uri: Uri) -> Response {
    let path = uri
        .path()
        .split('?')
        .next()
        .unwrap_or(uri.path())
        .trim_start_matches('/');

    if path.contains("..") || path.starts_with('/') {
        return (StatusCode::BAD_REQUEST, "Bad request").into_response();
    }

    let rel = if path.is_empty() { "index.html" } else { path };
    let file = state.dist.join(rel);
    match tokio::fs::read(&file).await {
        Ok(body) => {
            let mut response = Response::new(Body::from(body));
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(content_type(&file)),
            );
            response
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            (StatusCode::NOT_FOUND, "Not found").into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to read {}: {err}", file.display()),
        )
            .into_response(),
    }
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn spawn_jet_browser(fixture: &Path, url: &str) -> Result<Child> {
    Command::new(env!("CARGO_BIN_EXE_jet"))
        .args(["browser", "launch", url])
        .current_dir(fixture)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn jet browser launch")
}

async fn wait_for_browser_session(fixture: &Path) -> Result<()> {
    let session = fixture.join(".jet/browser-session.json");
    for _ in 0..150 {
        if session.exists() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(anyhow!(
        "jet browser launch did not write {}",
        session.display()
    ))
}

fn read_child_stderr(child: &mut Child) -> String {
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stderr.take() {
        let _ = pipe.read_to_string(&mut stderr);
    }
    stderr
}

fn wait_child_exit(child: &mut Child, context: &str) -> Result<String> {
    for _ in 0..120 {
        if let Some(status) = child.try_wait()? {
            let stderr = read_child_stderr(child);
            if status.success() {
                return Ok(stderr);
            }
            return Err(anyhow!("{context} exited with {status}\nstderr={stderr}"));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(anyhow!("{context} did not exit after shutdown request"))
}

fn browser_eval_json(fixture: &Path, expression: &str) -> Result<Value> {
    let output = require_success(
        run_jet(fixture, ["browser", "eval", expression])?,
        "browser eval",
    )?;
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("parse browser eval output for {expression:?}"))
}

fn shutdown_browser(fixture: &Path, child: &mut Child) -> Result<String> {
    require_success(
        run_jet(fixture, ["browser", "shutdown"])?,
        "browser shutdown",
    )?;
    wait_child_exit(child, "jet browser launch")
}

/// Writes a fixture designed to exercise the #1993 entry-flatten interop
/// shim from both directions at once, plus the pre-existing split-build
/// runtime properties it must not regress:
///
/// - `helper.js` is flatten-safe (no eval/cycle, single entry-chunk
///   reference) and lands in the flat region.
/// - `unsafe.js` uses `eval` (fallback-ladder rung: "fails scope-hoist
///   safety") so it stays on the `__jet__` registry, and itself statically
///   imports `helper.js` too — proving a registry-residue module's
///   `require()` correctly reads a flattened module's export via the
///   `__jet__.cache` pre-seed shim.
/// - `index.js` (the flat entry root) calls `unsafeRender()` synchronously
///   at module top level — proving the reverse direction: a flat module's
///   top-level code resolving a registry-residue import via the flat
///   region's local `_r` -> `__jet__.require` fallback.
/// - `common.js` is imported by both the entry and the lazily-loaded
///   chunk, forcing the pre-existing (#1963) `ChunkType::Shared` promotion
///   — proving entry-static shared-chunk resolution still works with a
///   flattened entry remainder.
/// - `lazy.js` is dynamically imported (unconditionally, not click-gated)
///   — proving the lazy-chunk-loads-on-demand mechanism (#1931) still
///   works with a flattened entry remainder.
fn write_entry_flatten_interop_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import { helper } from './helper.js';
import { unsafeRender } from './unsafe.js';
import { common } from './common.js';

document.getElementById('root').innerHTML =
  '<div id="output">' + helper() + '|' + unsafeRender() + '|' + common() + '</div>' +
  '<div id="lazy-output"></div>';

import('./lazy.js').then((mod) => mod.default());
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("src/helper.js"),
        "export function helper() { return 'HELPER_FLAT_MARKER'; }\n",
    )
    .expect("write helper");
    fs::write(
        dir.join("src/unsafe.js"),
        r#"import { helper } from './helper.js';

export function unsafeRender() {
  // `eval` forces this module onto the __jet__ registry (fallback-ladder
  // rung: "fails scope-hoist safety") even though the rest of its body is
  // ordinary.
  eval('1');
  return 'UNSAFE_REGISTRY_MARKER:' + helper();
}
"#,
    )
    .expect("write unsafe");
    fs::write(
        dir.join("src/common.js"),
        "export function common() { return 'SHARED_CHUNK_MARKER'; }\n",
    )
    .expect("write common");
    fs::write(
        dir.join("src/lazy.js"),
        r#"import { common } from './common.js';

export default function lazy() {
  document.getElementById('lazy-output').textContent = 'LAZY_MARKER:' + common();
}
"#,
    )
    .expect("write lazy");
}

/// #1993 — the flatten path must not regress any existing split-build
/// runtime property, and its registry interop shim must actually work end
/// -to-end (not just at the `scope_hoist`/`splitting` unit level): boots in
/// a real browser, the lazy chunk still loads on demand (#1931), the
/// entry-static shared chunk still resolves (#1963), and a registry
/// -residue module reading a flattened module's export — and a flattened
/// module reading a registry-residue module's export — both get the right
/// value.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn entry_flatten_boots_lazy_shared_and_registry_residue_interop_in_real_browser() -> Result<()>
{
    if !common::chromium_available() {
        eprintln!(
            "skipping entry_flatten_boots_lazy_shared_and_registry_residue_interop_in_real_browser: \
             no Chromium available"
        );
        return Ok(());
    }

    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_entry_flatten_interop_fixture(fixture);

    require_success(
        run_jet(fixture, ["build", "--splitting"])?,
        "build --splitting (entry-flatten interop fixture)",
    )?;
    assert!(
        fixture.join("dist/index.html").exists(),
        "splitting build must emit dist/index.html"
    );
    let shared_js_exists = list_files_recursive(&fixture.join("dist/assets"))
        .into_iter()
        .any(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("shared.") && n.ends_with(".js"))
        });
    assert!(
        shared_js_exists,
        "fixture must force a promoted shared.<hash>.js chunk (common.js used by \
         both the entry and the lazy chunk)"
    );

    // Same hard-bound-timeout shape as `code_splitting.rs`'s #1931/#1963
    // real-browser smokes.
    let result = tokio::time::timeout(Duration::from_secs(60), async {
        let server = StaticDistServer::spawn(fixture)
            .await
            .context("serve dist over local HTTP")?;
        let url = server.url();
        let mut browser =
            spawn_jet_browser(fixture, &url).context("jet browser launch dist/index.html")?;
        wait_for_browser_session(fixture)
            .await
            .context("wait for browser session file")?;

        let mut output_text = None;
        for _ in 0..120 {
            let value = browser_eval_json(
                fixture,
                "document.getElementById('output') && document.getElementById('output').textContent",
            )
            .unwrap_or(Value::Null);
            if let Some(text) = value.as_str() {
                output_text = Some(text.to_string());
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        assert_eq!(
            output_text.as_deref(),
            Some("HELPER_FLAT_MARKER|UNSAFE_REGISTRY_MARKER:HELPER_FLAT_MARKER|SHARED_CHUNK_MARKER"),
            "app must boot rendering: the flattened helper's own export, the \
             registry-residue module's export (itself correctly reading the \
             flattened helper's export via the __jet__.cache pre-seed shim), \
             and the promoted shared chunk's export"
        );

        let mut lazy_text = None;
        for _ in 0..120 {
            let value = browser_eval_json(
                fixture,
                "document.getElementById('lazy-output') && document.getElementById('lazy-output').textContent",
            )
            .unwrap_or(Value::Null);
            if let Some(text) = value.as_str() {
                if !text.is_empty() {
                    lazy_text = Some(text.to_string());
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        assert_eq!(
            lazy_text.as_deref(),
            Some("LAZY_MARKER:SHARED_CHUNK_MARKER"),
            "the lazily-loaded chunk must still load on demand and resolve the \
             shared chunk's export with a flattened entry remainder"
        );

        let resources = browser_eval_json(
            fixture,
            "performance.getEntriesByType('resource').map((e) => e.name)",
        )
        .context("read resource timing")?;
        let names: Vec<String> = resources
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        assert!(
            names.iter().any(|n| n.contains("/assets/shared.")),
            "the shared chunk's own script request must actually occur: {names:?}"
        );
        assert!(
            names.iter().any(|n| n.contains("chunk-lazy")),
            "the lazy chunk's own script request must actually occur: {names:?}"
        );

        shutdown_browser(fixture, &mut browser).context("jet browser shutdown")?;
        Ok::<(), anyhow::Error>(())
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => {
            let _ = Command::new(env!("CARGO_BIN_EXE_jet"))
                .args(["browser", "shutdown"])
                .current_dir(fixture)
                .output();
            Err(anyhow!(
                "entry_flatten_boots_lazy_shared_and_registry_residue_interop_in_real_browser \
                 timed out after 60s"
            ))
        }
    }
}

// ── size assertion ───────────────────────────────────────────────────────────

/// Writes a fixture with a meaningfully-sized, fully flatten-safe entry
/// module cluster (four helper modules with real logic, each reachable
/// only from the entry, none eval/cycle/cross-chunk-referenced) plus one
/// dynamic import — required to route the build through
/// `Bundler::generate_split_bundle` at all, since a graph with zero
/// dynamic imports never reaches the split path (see that function's own
/// early-return doc comment, issue #1932). Used to measure the
/// flatten-on-by-default entry file against the `JET_NO_ENTRY_FLATTEN=1`
/// escape hatch on identical source.
fn write_entry_flatten_size_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import { greet } from './greet.js';
import { mathOps } from './math.js';
import { strings } from './strings.js';
import { formatting } from './formatting.js';

document.getElementById('root').innerHTML =
  '<div id="output">' +
  greet('world') + '|' + mathOps(4, 5) + '|' + strings('abc') + '|' + formatting(3.14159) +
  '</div>';

import('./lazy.js').then((mod) => mod.default());
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("src/greet.js"),
        r#"function joinParts(parts) {
  return parts.join('');
}

export function greet(name) {
  return joinParts(['Hello', ', ', name, '!']);
}
"#,
    )
    .expect("write greet");
    fs::write(
        dir.join("src/math.js"),
        r#"function add(a, b) { return a + b; }
function multiply(a, b) { return a * b; }
function square(a) { return multiply(a, a); }

export function mathOps(a, b) {
  return String(add(a, b) + square(a) - square(b));
}
"#,
    )
    .expect("write math");
    fs::write(
        dir.join("src/strings.js"),
        r#"function reverse(str) {
  return str.split('').reverse().join('');
}
function upper(str) {
  return str.toUpperCase();
}

export function strings(str) {
  return reverse(str) + '/' + upper(str);
}
"#,
    )
    .expect("write strings");
    fs::write(
        dir.join("src/formatting.js"),
        r#"function pad(value, width) {
  const str = String(value);
  return str.length >= width ? str : '0'.repeat(width - str.length) + str;
}

export function formatting(value) {
  return pad(Math.round(value * 100), 6);
}
"#,
    )
    .expect("write formatting");
    fs::write(
        dir.join("src/lazy.js"),
        "export default function lazy() { return 'LAZY'; }\n",
    )
    .expect("write lazy");
}

/// Size assertion (#1993): on a fixture whose entire entry-chunk module set
/// is flatten-safe, the default (flatten-on) build must be strictly
/// smaller than the `JET_NO_ENTRY_FLATTEN=1` escape hatch on the same
/// fixture and the same `--splitting` flag — only the entry-flatten choice
/// differs. Also pins the `JET_BUNDLE_TIMING=1` partition-count line's
/// shape, since the size delta is only meaningful evidence of the flatten
/// path itself running if the partition actually reports a non-empty
/// flattened set.
#[test]
fn entry_flatten_default_is_strictly_smaller_than_no_entry_flatten_escape_hatch() -> Result<()> {
    let temp_on = tempfile::tempdir().context("tempdir (flatten on)")?;
    let fixture_on = temp_on.path();
    write_entry_flatten_size_fixture(fixture_on);

    let output_on = Command::new(env!("CARGO_BIN_EXE_jet"))
        .args(["build", "--splitting"])
        .current_dir(fixture_on)
        .env("JET_BUNDLE_TIMING", "1")
        .output()
        .context("run jet build (flatten on)")?;
    let output_on = require_success(output_on, "build --splitting (flatten on)")?;
    let stderr_on = String::from_utf8_lossy(&output_on.stderr).into_owned();
    let flattened = parse_partition_count(&stderr_on, "flattened").with_context(|| {
        format!("JET_BUNDLE_TIMING must report a flattened= partition count: {stderr_on}")
    })?;
    let registry_on = parse_partition_count(&stderr_on, "registry").with_context(|| {
        format!("JET_BUNDLE_TIMING must report a registry= partition count: {stderr_on}")
    })?;
    assert!(
        flattened > 0,
        "fixture's entry modules are all flatten-safe; expected flattened > 0, got \
         flattened={flattened} registry={registry_on} (stderr={stderr_on})"
    );

    let entry_on = find_entry_file(&fixture_on.join("dist"));
    let size_on = fs::metadata(&entry_on)
        .with_context(|| format!("stat {}", entry_on.display()))?
        .len();

    let temp_off = tempfile::tempdir().context("tempdir (flatten off)")?;
    let fixture_off = temp_off.path();
    write_entry_flatten_size_fixture(fixture_off);

    let output_off = Command::new(env!("CARGO_BIN_EXE_jet"))
        .args(["build", "--splitting"])
        .current_dir(fixture_off)
        .env("JET_NO_ENTRY_FLATTEN", "1")
        .output()
        .context("run jet build (flatten off)")?;
    require_success(output_off, "build --splitting (flatten off)")?;

    let entry_off = find_entry_file(&fixture_off.join("dist"));
    let size_off = fs::metadata(&entry_off)
        .with_context(|| format!("stat {}", entry_off.display()))?
        .len();

    assert!(
        size_on < size_off,
        "flatten-on entry ({size_on} bytes, flattened={flattened} registry={registry_on}) \
         must be strictly smaller than the JET_NO_ENTRY_FLATTEN=1 entry ({size_off} bytes)"
    );

    Ok(())
}
// HANDWRITE-END
