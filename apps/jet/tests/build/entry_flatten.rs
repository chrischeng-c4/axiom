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
use jet::bundler::scope_hoist_opt::find_top_level_bare_require_call;
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

/// Normalizes an entry file's code for cross-build byte-identity
/// comparison: blanks the trailing `//# sourceMappingURL=...` comment
/// (whose filename embeds the entry's own content hash) and splits the
/// `__jet__.chunkManifest = {...}` assignment out into a separately
/// returned, parsed `Value` (with the code's own copy of that JSON blob
/// replaced by a fixed placeholder).
///
/// Needed because `build_chunk_manifest_js`'s `chunks`/`moduleChunks` key
/// order is not currently stable run-to-run for otherwise byte-identical
/// input — confirmed via back-to-back `--splitting` builds of the exact
/// same fixture with the survivor filter in the *same* state, producing
/// distinct manifest key orderings and thus distinct entry content hashes.
/// That non-determinism sits entirely upstream of this WI (traced as far
/// as `build_chunk_manifest_js` in `src/cli.rs`, whose input chunk-slice
/// order is not itself sorted) and is reported separately rather than
/// fixed here; this helper (duplicated from `tests/build/code_splitting.rs`
/// — see this file's module doc comment for why harness code here is
/// duplicated rather than shared) makes the byte-identity test below
/// robust to it while still proving genuine byte-identity for everything
/// the survivor filter can actually affect.
fn normalize_entry_code(code: &str) -> (String, Value) {
    let normalized_lines: String = code
        .lines()
        .map(|line| {
            if line.starts_with("//# sourceMappingURL=") {
                "//# sourceMappingURL=<<STRIPPED>>"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let marker = "__jet__.chunkManifest";
    let marker_idx = normalized_lines
        .rfind(marker)
        .unwrap_or_else(|| panic!("entry code missing {marker:?}: {normalized_lines}"));
    let after_marker = marker_idx + marker.len();
    let eq_rel = normalized_lines[after_marker..]
        .find('=')
        .unwrap_or_else(|| panic!("no '=' found after {marker:?}"));
    let value_start = after_marker + eq_rel + 1;
    let tail = &normalized_lines[value_start..];
    let mut stream = serde_json::Deserializer::from_str(tail).into_iter::<Value>();
    let value = stream
        .next()
        .unwrap_or_else(|| panic!("no JSON value found after {marker:?} ="))
        .unwrap_or_else(|e| panic!("invalid JSON after {marker:?} =: {e}"));
    let value_end = value_start + stream.byte_offset();

    let mut placeholder_code = String::with_capacity(normalized_lines.len());
    placeholder_code.push_str(&normalized_lines[..value_start]);
    placeholder_code.push_str("<<CHUNK_MANIFEST>>");
    placeholder_code.push_str(&normalized_lines[value_end..]);
    (placeholder_code, value)
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
        Self::spawn_dir(&fixture.join("dist")).await
    }

    /// Same as `spawn`, but serves an explicit directory rather than always
    /// `<fixture>/dist` — used by #2128's export-elision A/B smoke, which
    /// builds the same fixture twice (`-o <dir>`) to compare the default
    /// (elision-on) output against the `JET_NO_EXPORT_ELISION=1` escape
    /// hatch without 2 separate tempdirs.
    async fn spawn_dir(dist: &Path) -> Result<Self> {
        let dist = dist.to_path_buf();
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

/// WI #1995 round 4/5/6 — the pre-transform survivors-only transform filter
/// (default-on since round 6; opt out via `JET_NO_SURVIVOR_FILTER=1`) must
/// be byte-identical to the escape-hatch (filter off) build on the mixed
/// entry-flatten fixture (flatten-safe modules + eval-forced registry
/// residue + a promoted shared chunk + a lazy chunk in one graph): same
/// `dist/` file set, same bytes.
///
/// Uses `--no-minify` (`build_chunk_manifest_js` only emits strict,
/// fully-quoted JSON before the minifier rewrites unquoted-identifier
/// keys) and, for the top-level entry file specifically, compares
/// `normalize_entry_code`'s output rather than raw bytes — see this test's
/// and that helper's doc comments for why raw entry bytes are not
/// currently a stable target. Both builds run from *one* fixture directory
/// (via `-o`, not 2 separate tempdirs): `--no-minify` output keeps a
/// `// Module N: <absolute path>` comment inline in chunk JS (not just
/// `.map`), so 2 different tempdirs would make even chunk files
/// legitimately differ for a reason unrelated to this WI. Every other
/// `dist/` file (including `index.html`, whose `<script src>` follows the
/// entry's own expected-to-differ hash and is normalized the same way) is
/// compared byte-for-byte.
#[test]
fn entry_flatten_survivor_filter_is_byte_identical_to_escape_hatch() -> Result<()> {
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_entry_flatten_interop_fixture(fixture);

    fn build(fixture: &Path, out_dir: &str, no_filter: bool) -> Result<PathBuf> {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_jet"));
        cmd.args(["build", "--splitting", "--no-minify", "-o", out_dir])
            .current_dir(fixture);
        // WI #1995 round 6: the survivors-only filter is default-on —
        // `no_filter=true` here means "exercise the opt-out escape hatch"
        // (`JET_NO_SURVIVOR_FILTER=1`), so that's the branch that opts out.
        if no_filter {
            cmd.env("JET_NO_SURVIVOR_FILTER", "1");
        }
        let output = cmd
            .output()
            .context("run jet build --splitting --no-minify")?;
        require_success(
            output,
            if no_filter {
                "build --splitting --no-minify (JET_NO_SURVIVOR_FILTER=1)"
            } else {
                "build --splitting --no-minify (default, survivor filter on)"
            },
        )?;
        Ok(fixture.join(out_dir))
    }

    /// `(relative-path, absolute-path)` for every dist file, sorted by
    /// relative path so 2 output trees zip up aligned.
    fn sorted_entries(dist: &Path) -> Vec<(String, PathBuf)> {
        let mut entries: Vec<(String, PathBuf)> = list_files_recursive(dist)
            .into_iter()
            .map(|p| {
                let rel = p
                    .strip_prefix(dist)
                    .expect("file under dist")
                    .to_string_lossy()
                    .into_owned();
                (rel, p)
            })
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries
    }

    let dist_filtered = build(fixture, "dist-filtered", false)?;
    let dist_unfiltered = build(fixture, "dist-unfiltered", true)?;

    let is_entry = |rel: &str| -> bool {
        rel.rsplit('/')
            .next()
            .is_some_and(|name| name.starts_with("main."))
    };
    let (filtered_entry, filtered_rest): (Vec<_>, Vec<_>) = sorted_entries(&dist_filtered)
        .into_iter()
        .partition(|(rel, _)| is_entry(rel));
    let (unfiltered_entry, unfiltered_rest): (Vec<_>, Vec<_>) = sorted_entries(&dist_unfiltered)
        .into_iter()
        .partition(|(rel, _)| is_entry(rel));

    // Entry file (`main.<hash>.js` + `.js.map`): the content hash itself is
    // expected to differ (see doc comment) — require both builds to still
    // emit exactly one entry `.js` + one entry `.js.map`, and use the 2
    // (differing) filenames to normalize `index.html`'s `<script src>`
    // reference below.
    assert_eq!(
        filtered_entry.len(),
        2,
        "expected exactly main.<hash>.js + .js.map, got {filtered_entry:?}"
    );
    assert_eq!(
        unfiltered_entry.len(),
        2,
        "expected exactly main.<hash>.js + .js.map, got {unfiltered_entry:?}"
    );
    fn entry_js_path(entries: &[(String, PathBuf)]) -> &Path {
        entries
            .iter()
            .find(|(rel, _)| rel.ends_with(".js"))
            .map(|(_, p)| p.as_path())
            .expect("main.<hash>.js missing")
    }
    let entry_filtered_path = entry_js_path(&filtered_entry);
    let entry_unfiltered_path = entry_js_path(&unfiltered_entry);
    let entry_filtered_name = entry_filtered_path
        .file_name()
        .and_then(OsStr::to_str)
        .expect("entry filename utf-8")
        .to_string();
    let entry_unfiltered_name = entry_unfiltered_path
        .file_name()
        .and_then(OsStr::to_str)
        .expect("entry filename utf-8")
        .to_string();

    // Non-entry files (chunk JS + every .map + index.html): exact
    // relative-path set and byte-identical content. Both builds share one
    // fixture directory (see doc comment), so the
    // `// Module N: <absolute path>` comment `--no-minify` inlines into
    // chunk JS is identical either way. `index.html` alone gets its
    // `<script src="./main.<hash>.js">` reference normalized first, since
    // it necessarily follows the entry's own (expected-to-differ) hash.
    let filtered_rest_rel: Vec<&String> = filtered_rest.iter().map(|(r, _)| r).collect();
    let unfiltered_rest_rel: Vec<&String> = unfiltered_rest.iter().map(|(r, _)| r).collect();
    assert_eq!(
        filtered_rest_rel, unfiltered_rest_rel,
        "survivor filter must emit the same non-entry dist/ file set as the \
         escape-hatch (JET_NO_SURVIVOR_FILTER=1, filter off) build on the mixed entry-flatten fixture"
    );
    for ((rel, fp), (_, up)) in filtered_rest.iter().zip(unfiltered_rest.iter()) {
        let fb = fs::read(fp).with_context(|| format!("read {}", fp.display()))?;
        let ub = fs::read(up).with_context(|| format!("read {}", up.display()))?;
        if rel == "index.html" {
            let fs_text = String::from_utf8(fb).context("index.html must be utf-8")?;
            let us_text = String::from_utf8(ub).context("index.html must be utf-8")?;
            assert_eq!(
                fs_text.replace(&entry_filtered_name, "<<ENTRY>>"),
                us_text.replace(&entry_unfiltered_name, "<<ENTRY>>"),
                "dist/index.html must be byte-identical (modulo the entry's \
                 own content-hashed filename) between the survivor filter \
                 and the escape-hatch (JET_NO_SURVIVOR_FILTER=1, filter off) build"
            );
            continue;
        }
        assert_eq!(
            fb, ub,
            "dist/{rel} must be byte-identical between the survivor filter \
             and the escape-hatch (JET_NO_SURVIVOR_FILTER=1, filter off) build"
        );
    }

    let entry_filtered_code =
        fs::read_to_string(entry_filtered_path).context("read filtered entry")?;
    let entry_unfiltered_code =
        fs::read_to_string(entry_unfiltered_path).context("read unfiltered entry")?;
    let (entry_filtered_norm, manifest_filtered) = normalize_entry_code(&entry_filtered_code);
    let (entry_unfiltered_norm, manifest_unfiltered) = normalize_entry_code(&entry_unfiltered_code);
    assert_eq!(
        entry_filtered_norm, entry_unfiltered_norm,
        "entry file must be byte-identical outside the chunkManifest \
         assignment (and its own sourceMappingURL comment) between the \
         survivor filter and the escape-hatch (JET_NO_SURVIVOR_FILTER=1, filter off) build"
    );
    assert_eq!(
        manifest_filtered, manifest_unfiltered,
        "chunkManifest must be the same JSON value (key order ignored — see \
         normalize_entry_code's doc comment) between the survivor filter \
         and the escape-hatch (JET_NO_SURVIVOR_FILTER=1, filter off) build"
    );

    Ok(())
}

// ── #2128 export-binding elision A/B smoke ──────────────────────────────────

/// Writes a fixture that exercises both branches of #2128's same-chunk
/// export-binding elision (`bundler/scope_hoist_opt.rs`,
/// `elide_same_chunk_export_bindings`) at once:
///
/// - `producer.js`'s `getMarkerUtilityClass` is a `function`-declared named
///   export (the real-world MUI `getXUtilityClass` shape that regressed
///   `production_build_regression` pre-fix — see
///   `elide_same_chunk_export_bindings`'s block-scope guard and
///   `test_elide_same_chunk_export_bindings_cross_block_function_declaration_keeps`).
///   Each flattened module lives in its own `{ ... }` block, so a
///   `function` declaration is block-scoped there under ES-module
///   strict-mode semantics; it must stay on the exports-object indirection
///   in *both* builds below, never elided.
/// - `producer.js`'s default export is consumed through `bridge.js`, a
///   pass-through re-export module shaped so it survives the pre-existing
///   `collapse_pure_reexport_wrappers` pass (it re-exports the *value*
///   under a name that already has a distinct binding, and additionally
///   owns an export of its own — `bridgeNoop` — so it is not a "pure"
///   wrapper; the real-corpus analog is `@mui/utils/esm/deepmerge/index.js`,
///   `export { default } from './deepmerge'; export * from './deepmerge';`,
///   which the same pass also leaves standing). That leaves `bridge.js`'s
///   own `_mN.exports["default"] = producedValue;` assignment — a plain
///   `var`-declared identifier RHS — for #2128's pass to find and elide.
fn write_export_elision_ab_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import { render } from './consumer.js';

document.getElementById('root').innerHTML = '<div id="output">' + render() + '</div>';
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("src/producer.js"),
        r#"export default function computeValue() {
  return 'PRODUCED_MARKER';
}

export function getMarkerUtilityClass(slot) {
  return 'KEPT_MARKER:' + slot;
}
"#,
    )
    .expect("write producer");
    fs::write(
        dir.join("src/bridge.js"),
        r#"import producedValue from './producer.js';

export default producedValue;

export function bridgeNoop() {
  console.log('bridge side effect, never called');
}
"#,
    )
    .expect("write bridge");
    fs::write(
        dir.join("src/consumer.js"),
        r#"import producedValue from './bridge.js';
import { getMarkerUtilityClass } from './producer.js';

export function render() {
  return producedValue() + '|' + getMarkerUtilityClass('slot');
}
"#,
    )
    .expect("write consumer");
}

/// #2128 — the `JET_NO_EXPORT_ELISION=1` escape hatch must be behaviorally
/// identical to the default (elision-on) build, in a real browser, on a
/// fixture that exercises both an elision-eligible export (`bridge.js`'s
/// re-exported default, a plain `var`-declared identifier) and
/// `getMarkerUtilityClass`, a `function` declaration — block-scoped, per
/// the regression #2128's implementation found and fixed against the real
/// mui-visual corpus. Also a basic byte-diff sanity check: the elision-on
/// entry must never be larger than the escape-hatch entry.
///
/// #2132 extends this A/B with a third variant: `getMarkerUtilityClass`
/// is *also* a var-hoisting-conversion candidate (a top-level flat-region
/// `function` declaration), so with elision left on but
/// `JET_NO_FN_DECL_CONVERSION=1` set, it is the one export in this fixture
/// that stays block-scoped and therefore "always kept" by elision — still
/// correct, just less size-optimal than the full default pipeline.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn export_elision_hatch_ab_boots_identically_in_real_browser() -> Result<()> {
    if !common::chromium_available() {
        eprintln!(
            "skipping export_elision_hatch_ab_boots_identically_in_real_browser: \
             no Chromium available"
        );
        return Ok(());
    }

    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_export_elision_ab_fixture(fixture);

    const EXPECTED_OUTPUT: &str = "PRODUCED_MARKER|KEPT_MARKER:slot";

    async fn build_and_boot(
        fixture: &Path,
        out_dir: &str,
        no_elision: bool,
        no_fn_decl_conversion: bool,
    ) -> Result<(String, u64)> {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_jet"));
        cmd.args(["build", "-o", out_dir]).current_dir(fixture);
        let mut phase_parts: Vec<&str> = Vec::new();
        if no_elision {
            cmd.env("JET_NO_EXPORT_ELISION", "1");
            phase_parts.push("JET_NO_EXPORT_ELISION=1");
        }
        if no_fn_decl_conversion {
            cmd.env("JET_NO_FN_DECL_CONVERSION", "1");
            phase_parts.push("JET_NO_FN_DECL_CONVERSION=1");
        }
        let phase = if phase_parts.is_empty() {
            "build (default, elision + fn-decl-conversion on)".to_string()
        } else {
            format!("build ({})", phase_parts.join(" "))
        };
        let phase = phase.as_str();
        let output = cmd.output().context("run jet build")?;
        require_success(output, phase)?;

        let dist = fixture.join(out_dir);
        let entry = find_entry_file(&dist);
        let size = fs::metadata(&entry)
            .with_context(|| format!("stat {}", entry.display()))?
            .len();

        let result = tokio::time::timeout(Duration::from_secs(60), async {
            let server = StaticDistServer::spawn_dir(&dist)
                .await
                .context("serve dist over local HTTP")?;
            let url = server.url();
            let mut browser = spawn_jet_browser(fixture, &url)
                .context("jet browser launch dist/index.html")?;
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

            shutdown_browser(fixture, &mut browser).context("jet browser shutdown")?;
            output_text.ok_or_else(|| anyhow!("#output element never rendered ({phase})"))
        })
        .await;

        let text = match result {
            Ok(inner) => inner?,
            Err(_) => {
                let _ = Command::new(env!("CARGO_BIN_EXE_jet"))
                    .args(["browser", "shutdown"])
                    .current_dir(fixture)
                    .output();
                return Err(anyhow!(
                    "export_elision_hatch_ab_boots_identically_in_real_browser: \
                     {phase} timed out after 60s"
                ));
            }
        };
        Ok((text, size))
    }

    let (text_on, size_on) = build_and_boot(fixture, "dist-elision-on", false, false).await?;
    let (text_off, size_off) = build_and_boot(fixture, "dist-elision-off", true, false).await?;
    let (text_no_fn_conv, size_no_fn_conv) =
        build_and_boot(fixture, "dist-fn-decl-conversion-off", false, true).await?;

    assert_eq!(
        text_on, EXPECTED_OUTPUT,
        "elision-on build must render the elided bridged default export and \
         the elided function-declared export identically to source"
    );
    assert_eq!(
        text_off, EXPECTED_OUTPUT,
        "JET_NO_EXPORT_ELISION=1 escape hatch must render identically to the \
         elision-on build"
    );
    assert_eq!(
        text_no_fn_conv, EXPECTED_OUTPUT,
        "JET_NO_FN_DECL_CONVERSION=1 escape hatch must render identically to \
         the default build (getMarkerUtilityClass stays block-scoped and \
         elision-ineligible, but still correct)"
    );
    assert!(
        size_on < size_off,
        "elision-on entry ({size_on} bytes) must be strictly smaller than the \
         JET_NO_EXPORT_ELISION=1 entry ({size_off} bytes)"
    );
    assert!(
        size_on < size_no_fn_conv,
        "default entry ({size_on} bytes) must be strictly smaller than the \
         JET_NO_FN_DECL_CONVERSION=1 entry ({size_no_fn_conv} bytes): \
         disabling #2132's conversion must leave getMarkerUtilityClass's \
         export indirection in place, forfeiting #2128's elision for it"
    );

    Ok(())
}

/// #2132's conservative "no earlier-in-execution-order reference" safety
/// condition, on a fixture engineered to exercise it: `parity.js` declares
/// a mutually-recursive pair (`isOdd` calls `isEven`; `isEven` calls
/// `isOdd`) plus an exported `describeParity` that calls `isEven`. The
/// textually-first declaration (`isOdd`) has no earlier reference to
/// itself anywhere in the flat region, so it converts; `isEven` is
/// referenced by `isOdd`'s body *before* `isEven`'s own declaration is
/// reached, so it must stay a hoisted `function` declaration;
/// `describeParity` is declared after both and is only ever referenced
/// from the entry module's block, which sits later still in the
/// topologically-ordered flat region — so it converts too. `import()`s
/// `lazy.js` purely to make `--splitting` actually engage the
/// entry-flatten path (`generate_entry_flat_region`, #1993) — the second
/// of #2132's two `mod.rs` call sites — rather than falling back to the
/// ordinary (non-split) Phase 2 flattening path that
/// `export_elision_hatch_ab_boots_identically_in_real_browser` above
/// already exercises.
fn write_fn_decl_hoisting_order_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import { describeParity } from './parity.js';

document.getElementById('root').innerHTML = '<div id="output">' + describeParity(7) + '</div>';

import('./lazy.js').then((mod) => mod.default());
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("src/parity.js"),
        r#"function isOdd(n) {
  return n === 0 ? false : isEven(n - 1);
}

function isEven(n) {
  return n === 0 ? true : isOdd(n - 1);
}

export function describeParity(n) {
  return n + ':' + (isEven(n) ? 'even' : 'odd');
}
"#,
    )
    .expect("write parity");
    fs::write(
        dir.join("src/lazy.js"),
        "export default function lazy() { return 'LAZY'; }\n",
    )
    .expect("write lazy");
}

/// Real-browser proof that #2132's conversion boots correctly on
/// module-internal mutual recursion via the entry-flatten path, and a
/// white-box pin (the `JET_BUNDLE_TIMING` `entry-flatten/fn-decl-conversion:`
/// line) confirming the mixed converted/skipped outcome actually happened
/// rather than the fixture merely happening to boot some other way.
/// Expected counts (empirically confirmed against this exact fixture
/// before authoring this assertion): `converted=2` (`isOdd`,
/// `describeParity`) and `skipped_order=1` (`isEven`, referenced by
/// `isOdd`'s body before its own declaration is reached).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fn_decl_conversion_mutual_recursion_boots_correctly_in_real_browser() -> Result<()> {
    if !common::chromium_available() {
        eprintln!(
            "skipping fn_decl_conversion_mutual_recursion_boots_correctly_in_real_browser: \
             no Chromium available"
        );
        return Ok(());
    }

    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_fn_decl_hoisting_order_fixture(fixture);

    let output = Command::new(env!("CARGO_BIN_EXE_jet"))
        .args(["build", "--splitting"])
        .current_dir(fixture)
        .env("JET_BUNDLE_TIMING", "1")
        .output()
        .context("run jet build --splitting")?;
    let output = require_success(output, "build --splitting (fn-decl hoisting-order fixture)")?;
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    let conversion_line = stderr
        .lines()
        .find(|line| line.contains("entry-flatten/fn-decl-conversion:"))
        .with_context(|| {
            format!("no entry-flatten/fn-decl-conversion timing line in stderr: {stderr}")
        })?;
    assert!(
        conversion_line.contains("converted=2 skipped_order=1 skipped_shape=0"),
        "expected isOdd and describeParity to convert (converted=2) and \
         isEven to be skipped for order (skipped_order=1), got: {conversion_line}"
    );

    let dist = fixture.join("dist");

    let result = tokio::time::timeout(Duration::from_secs(60), async {
        let server = StaticDistServer::spawn_dir(&dist)
            .await
            .context("serve dist over local HTTP")?;
        let url = server.url();
        let mut browser = spawn_jet_browser(fixture, &url)
            .context("jet browser launch dist/index.html")?;
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

        shutdown_browser(fixture, &mut browser).context("jet browser shutdown")?;
        output_text.ok_or_else(|| anyhow!("#output element never rendered"))
    })
    .await;

    let text = match result {
        Ok(inner) => inner?,
        Err(_) => {
            let _ = Command::new(env!("CARGO_BIN_EXE_jet"))
                .args(["browser", "shutdown"])
                .current_dir(fixture)
                .output();
            return Err(anyhow!(
                "fn_decl_conversion_mutual_recursion_boots_correctly_in_real_browser \
                 timed out after 60s"
            ));
        }
    };

    assert_eq!(
        text, "7:odd",
        "mutually-recursive isOdd/isEven (7 is odd) must still compute \
         correctly when only the textually-first declarations (isOdd, \
         describeParity) are var-hoisted and isEven stays a hoisted \
         function declaration"
    );

    Ok(())
}

// ── #2205: default-interop thunk placement ─────────────────────────────────

/// Writes a fixture that forces *two* registry-resident default-export
/// modules (`regA.js`, `regB.js`, both `eval`-forced onto the `__jet__`
/// registry — same technique as `write_entry_flatten_interop_fixture`'s
/// `unsafe.js`), each default-imported from *two* distinct flat-resident
/// consumers (`index.js` and `consumer2.js`). `partition_entry_for_flatten`
/// (`scope_hoist.rs`) emits `registry_ids` in sorted numeric order, so
/// whichever of `regA`/`regB` gets the smaller module id is textually
/// followed by the other's own `__jet__.define(...)` block — reproducing
/// #2205's real-corpus evidence
/// (`...});var e=_r(1012).default||_r(1012);__jet__.define(1029,...`)
/// regardless of which specific id jet's graph walk assigns to which file.
/// A trivial dynamic import (`lazy.js`) is required to route the build
/// through `Bundler::generate_split_bundle` at all (issue #1932).
fn write_interop_thunk_registry_placement_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import A from './regA.js';
import B from './regB.js';
import { renderTwo } from './consumer2.js';

document.getElementById('root').innerHTML =
  '<div id="output">' + A() + '|' + B() + '|' + renderTwo() + '</div>';

import('./lazy.js');
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("src/regA.js"),
        r#"export default function A() {
  // `eval` forces this module onto the __jet__ registry (fallback-ladder
  // rung: "fails scope-hoist safety") even though the rest of its body is
  // ordinary.
  eval('1');
  return 'A_MARKER';
}
"#,
    )
    .expect("write regA");
    fs::write(
        dir.join("src/regB.js"),
        r#"export default function B() {
  eval('1');
  return 'B_MARKER';
}
"#,
    )
    .expect("write regB");
    fs::write(
        dir.join("src/consumer2.js"),
        r#"import A from './regA.js';
import B from './regB.js';

export function renderTwo() {
  return 'C2:' + A() + ':' + B();
}
"#,
    )
    .expect("write consumer2");
    fs::write(
        dir.join("src/lazy.js"),
        "export default function lazy() {}\n",
    )
    .expect("write lazy");
}

/// #2205 — a default-interop thunk hoisted by `hoist_default_interop_thunks`
/// (`scope_hoist_opt.rs`) for a *registry*-resident id must never land
/// outside the flat region's `_r`-scoped IIFE (`ReferenceError: _r is not
/// defined`, silent black page in production while `jet build` reports
/// success). Asserts both the cheap static invariant (no bare `_r(` call at
/// true top level anywhere in the built entry) and the actual product
/// behavior (the app boots and renders correctly in a real browser).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn entry_flatten_default_interop_thunk_for_registry_resident_id_stays_in_scope() -> Result<()>
{
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_interop_thunk_registry_placement_fixture(fixture);

    let output = Command::new(env!("CARGO_BIN_EXE_jet"))
        .args(["build", "--splitting"])
        .current_dir(fixture)
        .env("JET_BUNDLE_TIMING", "1")
        .output()
        .context("run jet build --splitting")?;
    let output = require_success(
        output,
        "build --splitting (interop thunk registry placement fixture)",
    )?;
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    // Sanity: the fixture must actually exercise the #1993 combined
    // registry+flat path with >= 2 registry-resident modules (regA, regB)
    // — otherwise this test would silently stop covering the bug class if
    // some upstream change ever widened flatten-safety and swallowed the
    // `eval` fallback-ladder rung.
    let registry_count = parse_partition_count(&stderr, "registry");
    assert!(
        matches!(registry_count, Some(n) if n >= 2),
        "expected >= 2 registry-resident modules (regA.js + regB.js, both \
         eval-forced), got {registry_count:?}\nstderr={stderr}"
    );

    let dist = fixture.join("dist");
    let entry_path = find_entry_file(&dist);
    let entry_code = fs::read_to_string(&entry_path).context("read built entry file")?;

    // (a) Static guard (issue #2205 permanent regression check, cheap and
    // no browser required): the built entry must never contain a bare
    // `_r(` call at true top level, outside every `__jet__.define(...)`
    // factory and outside the flat region's own IIFE.
    assert!(
        find_top_level_bare_require_call(&entry_code).is_none(),
        "found a default-interop thunk (or other `_r(` call) outside any \
         scope that declares `_r` in the built entry — see \
         `hoist_default_interop_thunks`'s doc comment (#2205)\n{entry_code}"
    );

    // (b) Dynamic proof: the app must actually boot and render in a real
    // Chromium, not merely pass the static shape check.
    if !common::chromium_available() {
        eprintln!(
            "skipping real-browser half of \
             entry_flatten_default_interop_thunk_for_registry_resident_id_stays_in_scope: \
             no Chromium available"
        );
        return Ok(());
    }

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

        shutdown_browser(fixture, &mut browser).context("jet browser shutdown")?;
        Ok::<Option<String>, anyhow::Error>(output_text)
    })
    .await;

    let output_text = match result {
        Ok(inner) => inner?,
        Err(_) => {
            let _ = Command::new(env!("CARGO_BIN_EXE_jet"))
                .args(["browser", "shutdown"])
                .current_dir(fixture)
                .output();
            return Err(anyhow!(
                "entry_flatten_default_interop_thunk_for_registry_resident_id_stays_in_scope \
                 timed out after 60s"
            ));
        }
    };

    assert_eq!(
        output_text.as_deref(),
        Some("A_MARKER|B_MARKER|C2:A_MARKER:B_MARKER"),
        "app must boot rendering both registry-resident default exports \
         (read from the entry's own top level and from a second flat \
         consumer) without a `ReferenceError: _r is not defined` black page"
    );

    Ok(())
}

fn write_interop_thunk_section_boundary_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("index.html"),
        "<!doctype html><html><body><div id=\"root\"></div>\
         <script type=\"module\" src=\"/src/index.js\"></script></body></html>",
    )
    .expect("write index.html");
    fs::write(
        dir.join("src/index.js"),
        r#"import T from './target.js';
import { useT } from './helper.js';

document.getElementById('root').innerHTML =
  '<div id="output">' + (T() + useT()) + '</div>';

import('./lazy.js');
"#,
    )
    .expect("write entry");
    // The leading comment is an ordinary source comment, not a jet banner —
    // it is deliberately shaped like the loose (pre-#2205-round-2) section
    // boundary match `"\n// Module "` while lacking the digits-plus-colon
    // suffix a real `// Module <id>: <path>` banner always has.
    fs::write(
        dir.join("src/target.js"),
        r#"// Module for target utilities
export default function T() {
  return 42;
}
"#,
    )
    .expect("write target");
    fs::write(
        dir.join("src/helper.js"),
        r#"import T from './target.js';

export function useT() {
  return T() + 1;
}
"#,
    )
    .expect("write helper");
    fs::write(dir.join("src/lazy.js"), "export const lazyValue = 99;\n").expect("write lazy");
}

/// #2205 round 2 — `hoist_default_interop_thunks`'s per-module section
/// boundary search (`section_end`, `scope_hoist_opt.rs`) used a loose
/// `"\n// Module "` prefix match to find where a flat-resident module's
/// own inlined body ends. This pass runs pre-minify, so a flat module's
/// own *original source comment* shaped like an ordinary `// Module ...`
/// phrasing is byte-identical to that loose prefix and falsely ends the
/// section early — inside the module's own still-executing block, before
/// its function declaration and export assignment run. The hoisted
/// `_di{id}` then reads that module's own not-yet-populated exports
/// (self-referential), producing a value every consumer chokes on, which
/// throws inside the minifier's comma-joined top-level statement sequence
/// and silently aborts every statement after it — including the trailing
/// `__jet__.require(<entry>)` boot call (`jet build` reports success; the
/// production page renders a blank/black root with zero console output).
/// Fixed by requiring the full banner shape (`\n// Module \d+: `, matching
/// `format!("// Module {}: {}\n", id, path)` byte-for-byte) instead of the
/// loose 11-byte prefix.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn entry_flatten_default_interop_thunk_section_boundary_skips_lookalike_source_comment(
) -> Result<()> {
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_interop_thunk_section_boundary_fixture(fixture);

    let output = Command::new(env!("CARGO_BIN_EXE_jet"))
        .args(["build", "--splitting"])
        .current_dir(fixture)
        .env("JET_BUNDLE_TIMING", "1")
        .env("JET_NO_PERSISTENT_CACHE", "1")
        .output()
        .context("run jet build --splitting")?;
    let output = require_success(
        output,
        "build --splitting (interop thunk section boundary fixture)",
    )?;
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    // Sanity: the fixture must actually exercise the entry-flatten path
    // with target.js/helper.js/index.js all flat-resident — otherwise this
    // test would silently stop covering the bug class.
    let flattened_count = parse_partition_count(&stderr, "flattened");
    assert!(
        matches!(flattened_count, Some(n) if n >= 3),
        "expected >= 3 flat-resident modules (index.js + target.js + \
         helper.js), got {flattened_count:?}\nstderr={stderr}"
    );

    let dist = fixture.join("dist");
    let entry_path = find_entry_file(&dist);
    let entry_code = fs::read_to_string(&entry_path).context("read built entry file")?;

    // (a) Static guard (#2205 round 1's class, cheap and no browser
    // required): the built entry must never contain a bare `_r(` call at
    // true top level. Kept alongside (b) since it does not by itself catch
    // round 2's depth > 0 shape (see `find_top_level_bare_require_call`'s
    // doc comment) — the real-browser boot proof below is this bug class's
    // actual guard.
    assert!(
        find_top_level_bare_require_call(&entry_code).is_none(),
        "found a default-interop thunk (or other `_r(` call) outside any \
         scope that declares `_r` in the built entry — see \
         `hoist_default_interop_thunks`'s doc comment (#2205)\n{entry_code}"
    );

    // (b) Dynamic proof: the app must actually boot and render in a real
    // Chromium, not merely pass the static shape check.
    if !common::chromium_available() {
        eprintln!(
            "skipping real-browser half of \
             entry_flatten_default_interop_thunk_section_boundary_skips_lookalike_source_comment: \
             no Chromium available"
        );
        return Ok(());
    }

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

        shutdown_browser(fixture, &mut browser).context("jet browser shutdown")?;
        Ok::<Option<String>, anyhow::Error>(output_text)
    })
    .await;

    let output_text = match result {
        Ok(inner) => inner?,
        Err(_) => {
            let _ = Command::new(env!("CARGO_BIN_EXE_jet"))
                .args(["browser", "shutdown"])
                .current_dir(fixture)
                .output();
            return Err(anyhow!(
                "entry_flatten_default_interop_thunk_section_boundary_skips_lookalike_source_comment \
                 timed out after 60s"
            ));
        }
    };

    assert_eq!(
        output_text.as_deref(),
        Some("85"),
        "app must boot rendering T() + useT() = 42 + 43 = 85 without the \
         hoisted interop thunk landing inside target.js's own still- \
         executing block and reading its not-yet-populated exports"
    );

    Ok(())
}

// ── nested exports-map conditions (#2261) ───────────────────────────────────
//
// react-router@7.18.1's `./dom` subpath nests two levels of condition
// objects (outer node/module/import/default, inner
// types/module-sync/default or types/default). Graph-walk resolution
// (`resolver::package::resolve_export_value`) already recursed correctly
// through this shape; the actual #2261 bug was codegen's *independent*
// re-derivation of "bare specifier text -> module id"
// (`resolve_bare_specifier_from_index` in `transform::modules`), which only
// tried a literal `root.join(subpath)` disk join and never consulted the
// exports map at all — silently falling through to a naked string
// `require('rr-core/dom')` that throws `Module not found` at runtime
// (jet's `__jet__.require` is purely numeric-id-keyed; see
// `Bundler::generate_runtime`) while `jet build` reported success.

/// Writes an `rr-shim` -> `rr-core/dom` package chain shaped exactly like
/// the real-corpus react-router@7.18.1 regression (#2261): `rr-core`'s
/// `package.json` nests its `"./dom"` export two levels deep (outer
/// node/module/import/default, inner types/default), and `rr-shim`
/// re-exports it wholesale (`export * from 'rr-core/dom'`) the same way
/// react-router's own `index.js` re-exports from `./dom`. The active
/// browser-production conditions (`browser`, `module`, `import`,
/// `production`, `default` — `ResolveOptions::for_browser_production`)
/// select the outer `"module"` key, then its inner `"default"` key,
/// landing on `dist/development/dom-export.mjs`. A dynamic `import()` of an
/// unrelated `lazy.js` is included so `--splitting` actually routes through
/// the entry-flatten path (`Bundler::generate_split_bundle`'s early return
/// on zero dynamic imports, issue #1932) rather than the plain single
/// -bundle path.
fn write_nested_exports_map_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("node_modules/rr-core/dist/development"))
        .expect("create node_modules/rr-core/dist/development");
    fs::write(
        dir.join("node_modules/rr-core/package.json"),
        r#"{
  "name": "rr-core",
  "version": "1.0.0",
  "exports": {
    "./dom": {
      "node": {
        "types": "./dist/development/dom-export.d.ts",
        "module": "./dist/development/dom-export.mjs",
        "module-sync": "./dist/development/dom-export.js",
        "default": "./dist/development/dom-export.js"
      },
      "module": {
        "types": "./dist/development/dom-export.d.ts",
        "default": "./dist/development/dom-export.mjs"
      },
      "import": {
        "types": "./dist/development/dom-export.d.ts",
        "default": "./dist/development/dom-export.mjs"
      },
      "default": "./dist/development/dom-export.js"
    }
  }
}
"#,
    )
    .expect("write rr-core package.json");
    fs::write(
        dir.join("node_modules/rr-core/dist/development/dom-export.mjs"),
        "export const RR_DOM_MARKER = 'RR_DOM_MARKER_VALUE';\n",
    )
    .expect("write rr-core dom-export.mjs (module/import condition target)");
    // The node/default condition target also exists so a resolver
    // misconfiguration that selects the wrong branch fails loudly (reading
    // the wrong marker) rather than passing by accident.
    fs::write(
        dir.join("node_modules/rr-core/dist/development/dom-export.js"),
        "exports.RR_DOM_MARKER = 'RR_DOM_MARKER_VALUE_NODE';\n",
    )
    .expect("write rr-core dom-export.js (node/default condition target)");

    fs::create_dir_all(dir.join("node_modules/rr-shim")).expect("create node_modules/rr-shim");
    // `"sideEffects": false` (a common, realistic declaration for a pure
    // re-export shim, matching real react-router's own package.json) keeps
    // rr-shim itself in the flat region alongside rr-core's resolved
    // target: without it, `is_entry_module_flatten_safe`'s conservative
    // `.js`-file-in-`node_modules` heuristic (`is_side_effect_free`,
    // scope_hoist.rs) leaves rr-shim on the `__jet__` registry, which would
    // instead exercise the *separate*, already-documented #1993 residual
    // limitation ("Residual limitation" doc comment above
    // `generate_entry_flat_region`): a flat module's synchronous top-level
    // call into registry residue that itself requires a flat module not
    // otherwise directly imported by the caller can race the flat-only
    // topological order. That gap is orthogonal to #2261's resolver bug
    // and intentionally left untouched here — this fixture avoids it
    // entirely so the real-browser assertion below is a clean read on the
    // exports-map fix alone.
    fs::write(
        dir.join("node_modules/rr-shim/package.json"),
        r#"{"name":"rr-shim","version":"1.0.0","main":"index.js","sideEffects":false}"#,
    )
    .expect("write rr-shim package.json");
    fs::write(
        dir.join("node_modules/rr-shim/index.js"),
        "export * from 'rr-core/dom';\n",
    )
    .expect("write rr-shim index.js");

    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import { RR_DOM_MARKER } from 'rr-shim';

document.getElementById('root').innerHTML = '<div id="output">' + RR_DOM_MARKER + '</div>';

import('./lazy.js');
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("src/lazy.js"),
        "export default function lazy() { return 'LAZY'; }\n",
    )
    .expect("write lazy");
}

/// Scans every built `.js` file under `dist` for a literal occurrence of
/// `needle` — the cheap, no-browser-required static-guard half of this
/// test (the same "(a) Static guard" / "(b) Dynamic proof" two-step shape
/// `entry_flatten_default_interop_thunk_section_boundary_skips_lookalike_source_comment`
/// above uses, extended here to scan the whole `dist` tree rather than
/// just the entry file since a resolved-vs-fallback subpath could in
/// principle land in either).
fn dist_js_files_containing(dist: &Path, needle: &str) -> Vec<PathBuf> {
    list_files_recursive(dist)
        .into_iter()
        .filter(|p| p.extension().and_then(OsStr::to_str) == Some("js"))
        .filter(|p| fs::read_to_string(p).is_ok_and(|code| code.contains(needle)))
        .collect()
}

/// #2261 — a package's `exports` map may nest condition objects two (or
/// more) levels deep (react-router@7.18.1's `./dom` subpath: outer
/// node/module/import/default, inner types/module-sync/default). Codegen's
/// independent bare-specifier re-derivation
/// (`resolve_bare_specifier_from_index` in `transform::modules`) must
/// resolve through that nesting the same way graph-walk resolution does —
/// not silently fall through to a naked string `require('rr-core/dom')`
/// that throws `Module not found` at runtime while `jet build` reports
/// success (a black page in production).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn entry_flatten_resolves_nested_exports_map_condition_subpath_through_reexport() -> Result<()>
{
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_nested_exports_map_fixture(fixture);

    require_success(
        run_jet(fixture, ["build", "--splitting"])?,
        "build --splitting (nested exports-map condition fixture)",
    )?;
    assert!(
        fixture.join("dist/index.html").exists(),
        "splitting build must emit dist/index.html"
    );

    let dist = fixture.join("dist");

    // (a) Static guard (cheap, no browser required): the raw specifier text
    // must never survive codegen as a naked string require — that is
    // exactly the #2261 symptom (a numeric-id-only `__jet__.require`
    // runtime throwing `Module not found` for a string id at page load).
    let offending = dist_js_files_containing(&dist, "require('rr-core/dom')");
    assert!(
        offending.is_empty(),
        "found a naked string require('rr-core/dom') in built output (the \
         #2261 bug: codegen fell through to a runtime string require \
         instead of resolving the nested exports-map condition target) in: \
         {offending:?}"
    );

    // (b) Dynamic proof: the app must actually boot and render in a real
    // Chromium, not merely pass the static shape check.
    if !common::chromium_available() {
        eprintln!(
            "skipping real-browser half of \
             entry_flatten_resolves_nested_exports_map_condition_subpath_through_reexport: \
             no Chromium available"
        );
        return Ok(());
    }

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

        shutdown_browser(fixture, &mut browser).context("jet browser shutdown")?;
        Ok::<Option<String>, anyhow::Error>(output_text)
    })
    .await;

    let output_text = match result {
        Ok(inner) => inner?,
        Err(_) => {
            let _ = Command::new(env!("CARGO_BIN_EXE_jet"))
                .args(["browser", "shutdown"])
                .current_dir(fixture)
                .output();
            return Err(anyhow!(
                "entry_flatten_resolves_nested_exports_map_condition_subpath_through_reexport \
                 timed out after 60s"
            ));
        }
    };

    assert_eq!(
        output_text.as_deref(),
        Some("RR_DOM_MARKER_VALUE"),
        "app must boot rendering the re-exported nested-condition target's \
         value instead of failing to load with a black page (#2261)"
    );

    Ok(())
}

// ── multi-line bare export lists through a star re-export (#2261 round 3) ──
//
// react-router@7.18.1's own built `dist/development/index.mjs` re-exports
// every public name it only ever *imports* (never locally declares) via one
// long, bare (no `from` clause) `export { ... };` list — prettier's default
// wrapping once the list is long enough (react-router's real list spans 130
// lines). `tree_shake::extract_export_names` (feeding
// `RawModuleFacts::exports`, the analysis layer's only view of "what does
// this module export") scanned `export { ... }` one physical source line at
// a time: every continuation line after `export {` matched neither the
// single-line name-extraction branch nor found a `}` on that same line, so
// the entire multi-line block silently contributed zero names. With
// `all_exports[barrel-core] == []`, the star-reexport propagation fixed
// point in `analyze_used_exports_from_with_raw_facts_provider` (for
// `barrel-dom`'s `export * from 'barrel-core';`) always computed an empty
// `leaf_exports ∩ barrel_used` intersection and never created a
// `used_exports[barrel-core]` entry, so `compute_transform_survivors`
// dropped `barrel-core` from the survivors set entirely — even though
// `barrel-dom`'s compiled glue still contains a numeric `require(<id>)` call
// into it (proving it's genuinely reachable). Round 3's
// `Bundler::check_referenced_module_emission` consistency guard turns that
// silent drop into a loud build failure; the real fix is
// `extract_export_names` accumulating continuation lines until the closing
// `}` is found (the same pattern `logical_import_lines` already uses for
// multi-line `import { ... } from '...'`).
//
// Unlike the neighboring nested-exports-map fixture above, this bug lives in
// tree-shaking's analysis pre-pass rather than the entry-flatten path, so a
// plain `jet build` (no `--splitting`) reproduces it directly.

/// Writes a `barrel-dom` -> `barrel-core` package chain shaped like
/// react-router-dom -> react-router's real built output: `barrel-core`'s
/// `index.js` imports two functions from a sibling chunk, then re-exports
/// them via one **bare, multi-line** `export { ... };` list (no `from`
/// clause, closing `}` on its own line — prettier's wrapping once a list
/// has 3+ names) instead of declaring them locally. `barrel-dom` re-exports
/// the whole thing wildcard-style (`export * from 'barrel-core';`), exactly
/// like react-router-dom's `export * from "react-router"`. The app entry
/// imports the two names through `barrel-dom` and *calls* them (not just
/// references them) so the real-browser assertion proves genuine runtime
/// wiring, not merely that the build didn't crash.
fn write_multiline_bare_export_barrel_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("node_modules/barrel-core")).expect("create barrel-core");
    fs::write(
        dir.join("node_modules/barrel-core/package.json"),
        r#"{"name":"barrel-core","version":"1.0.0","main":"index.js","sideEffects":false}"#,
    )
    .expect("write barrel-core package.json");
    fs::write(
        dir.join("node_modules/barrel-core/chunk.js"),
        r#"export function greet(name) {
  return 'HELLO_' + name;
}
export function farewell(name) {
  return 'BYE_' + name;
}
"#,
    )
    .expect("write barrel-core chunk.js");
    // The bug: `greet`/`farewell` are never declared in this file — they are
    // only imported, then re-exported via a bare (no `from`) list whose
    // closing `}` lives on its own physical line.
    fs::write(
        dir.join("node_modules/barrel-core/index.js"),
        r#"import {
  greet,
  farewell
} from './chunk.js';

export {
  greet,
  farewell
};
"#,
    )
    .expect("write barrel-core index.js");

    fs::create_dir_all(dir.join("node_modules/barrel-dom")).expect("create barrel-dom");
    fs::write(
        dir.join("node_modules/barrel-dom/package.json"),
        r#"{"name":"barrel-dom","version":"1.0.0","main":"index.js","sideEffects":false}"#,
    )
    .expect("write barrel-dom package.json");
    fs::write(
        dir.join("node_modules/barrel-dom/index.js"),
        "export * from 'barrel-core';\n",
    )
    .expect("write barrel-dom index.js");

    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import { greet, farewell } from 'barrel-dom';

document.getElementById('root').innerHTML =
  '<div id="output">' + greet('WORLD') + '_' + farewell('WORLD') + '</div>';
"#,
    )
    .expect("write entry");
}

/// #2261 round 3 — a package's only public surface may be a **bare,
/// multi-line** `export { ... };` re-export list (no `from` clause, closing
/// `}` on its own line) rather than named local declarations or a
/// single-line list. Tree-shaking's analysis pre-pass
/// (`tree_shake::extract_export_names`) must still see every name so a
/// downstream `export * from '...'` barrel can propagate usage through it —
/// not silently record zero exports and let `compute_transform_survivors`
/// drop the module while other compiled code still calls into it by
/// numeric id (a referenced-but-never-emitted module, caught loudly by
/// `Bundler::check_referenced_module_emission` rather than crashing at
/// runtime with a black page).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn entry_flatten_resolves_multiline_bare_export_list_via_star_reexport() -> Result<()> {
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_multiline_bare_export_barrel_fixture(fixture);

    // The build itself must succeed: pre-fix, `barrel-core` is dropped from
    // the survivors set while `barrel-dom`'s compiled glue still calls into
    // it by numeric id, and `check_referenced_module_emission` turns that
    // into a loud build failure rather than a silent black page.
    require_success(
        run_jet(fixture, ["build"])?,
        "build (multi-line bare export barrel fixture)",
    )?;
    assert!(
        fixture.join("dist/index.html").exists(),
        "build must emit dist/index.html"
    );

    let dist = fixture.join("dist");

    // (a) Static guard (module presence, cheap, no browser required):
    // `barrel-core`'s real function bodies must have been emitted, not just
    // referenced — the exact "referenced but never emitted" failure this
    // bug produced before the analysis fix (and that the consistency guard
    // above would otherwise have to catch at build time).
    let emitted = dist_js_files_containing(&dist, "HELLO_");
    assert!(
        !emitted.is_empty(),
        "expected barrel-core's greet() body ('HELLO_') to be emitted \
         somewhere in dist — the module must survive tree-shaking's \
         analysis pre-pass, not just be referenced by numeric id from \
         barrel-dom's compiled glue"
    );

    // (b) Dynamic proof: the app must actually boot and render in a real
    // Chromium, proving the re-exported functions are correctly wired
    // through the star-reexport barrel, not merely present as inert text.
    if !common::chromium_available() {
        eprintln!(
            "skipping real-browser half of \
             entry_flatten_resolves_multiline_bare_export_list_via_star_reexport: \
             no Chromium available"
        );
        return Ok(());
    }

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

        shutdown_browser(fixture, &mut browser).context("jet browser shutdown")?;
        Ok::<Option<String>, anyhow::Error>(output_text)
    })
    .await;

    let output_text = match result {
        Ok(inner) => inner?,
        Err(_) => {
            let _ = Command::new(env!("CARGO_BIN_EXE_jet"))
                .args(["browser", "shutdown"])
                .current_dir(fixture)
                .output();
            return Err(anyhow!(
                "entry_flatten_resolves_multiline_bare_export_list_via_star_reexport \
                 timed out after 60s"
            ));
        }
    };

    assert_eq!(
        output_text.as_deref(),
        Some("HELLO_WORLD_BYE_WORLD"),
        "app must boot rendering both functions re-exported through the \
         bare multi-line export list and the star-reexport barrel \
         (#2261 round 3)"
    );

    Ok(())
}

fn write_cjs_entry_shim_specifier_collision_fixture(dir: &Path) {
    // Hoisted copy: a react-shaped 2-line conditional CJS entry shim (no
    // ESM export statements at all) sitting at the conventional top-level
    // `node_modules/pkg/index.js` location every *external* consumer's
    // real (importer-aware) resolution should land on.
    fs::create_dir_all(dir.join("node_modules/pkg/cjs")).expect("create node_modules/pkg/cjs");
    fs::write(
        dir.join("node_modules/pkg/package.json"),
        r#"{"name":"pkg","version":"1.0.0","main":"index.js","sideEffects":false}"#,
    )
    .expect("write pkg package.json");
    fs::write(
        dir.join("node_modules/pkg/index.js"),
        r#"if (process.env.NODE_ENV === 'production') {
  module.exports = require('./cjs/pkg.production.js');
} else {
  module.exports = require('./cjs/pkg.development.js');
}
"#,
    )
    .expect("write pkg index.js");
    fs::write(
        dir.join("node_modules/pkg/cjs/pkg.production.js"),
        "exports.MARKER = 'HOISTED_PKG';\n",
    )
    .expect("write pkg cjs/pkg.production.js");
    fs::write(
        dir.join("node_modules/pkg/cjs/pkg.development.js"),
        "exports.MARKER = 'HOISTED_PKG_DEV';\n",
    )
    .expect("write pkg cjs/pkg.development.js");

    // `other-lib` carries its own non-hoisted, nested duplicate of `pkg`
    // under its own `node_modules/` (a realistic version-conflict shape) —
    // `other-lib`'s own `require('pkg')` legitimately resolves to *this*
    // copy via real nearest-wins node_modules walk-up, not the top-level
    // hoisted one.
    fs::create_dir_all(dir.join("node_modules/other-lib/node_modules/pkg/cjs"))
        .expect("create other-lib's nested node_modules/pkg/cjs");
    fs::write(
        dir.join("node_modules/other-lib/package.json"),
        r#"{"name":"other-lib","version":"1.0.0","main":"index.js","sideEffects":false}"#,
    )
    .expect("write other-lib package.json");
    fs::write(
        dir.join("node_modules/other-lib/index.js"),
        "const pkg = require('pkg');\nmodule.exports = { otherLibMarker: pkg.MARKER };\n",
    )
    .expect("write other-lib index.js");
    // A *different* version string than the hoisted copy's `1.0.0` is
    // load-bearing: `resolver::resolve_package`'s walk-up has a
    // `hoisted_same_version` dedup rule that prefers the outermost copy
    // whenever an outer candidate's version string exactly matches the
    // nearest one — which would silently collapse this fixture's intended
    // two-target split back down to a single (hoisted-only) target even
    // for `other-lib`'s own nested-tree-local `require('pkg')`.
    fs::write(
        dir.join("node_modules/other-lib/node_modules/pkg/package.json"),
        r#"{"name":"pkg","version":"2.0.0","main":"index.js","sideEffects":false}"#,
    )
    .expect("write other-lib's nested pkg package.json");
    fs::write(
        dir.join("node_modules/other-lib/node_modules/pkg/index.js"),
        r#"if (process.env.NODE_ENV === 'production') {
  module.exports = require('./cjs/pkg.production.js');
} else {
  module.exports = require('./cjs/pkg.development.js');
}
"#,
    )
    .expect("write other-lib's nested pkg index.js");
    fs::write(
        dir.join("node_modules/other-lib/node_modules/pkg/cjs/pkg.production.js"),
        "exports.MARKER = 'NESTED_PKG';\n",
    )
    .expect("write other-lib's nested pkg cjs/pkg.production.js");
    fs::write(
        dir.join("node_modules/other-lib/node_modules/pkg/cjs/pkg.development.js"),
        "exports.MARKER = 'NESTED_PKG_DEV';\n",
    )
    .expect("write other-lib's nested pkg cjs/pkg.development.js");

    // Five more external consumers (outside `other-lib`'s own tree) doing a
    // plain bare-specifier default import — standing in for the real
    // corpus's ~180-file fan-in (every one of these must resolve to the
    // *hoisted* copy, exactly like `other-lib`'s own import must keep
    // resolving to its *nested* copy).
    for n in 1..=5 {
        let pkg_dir = format!("node_modules/consumer{n}");
        fs::create_dir_all(dir.join(&pkg_dir)).expect("create consumerN dir");
        fs::write(
            dir.join(format!("{pkg_dir}/package.json")),
            format!(
                r#"{{"name":"consumer{n}","version":"1.0.0","main":"index.js","sideEffects":false}}"#
            ),
        )
        .expect("write consumerN package.json");
        fs::write(
            dir.join(format!("{pkg_dir}/index.js")),
            "import pkg from 'pkg';\nexport default pkg.MARKER;\n",
        )
        .expect("write consumerN index.js");
    }

    // Entry: the *hoisted* `pkg` import is textually first (so it is
    // pushed to the crawl stack first) and `other-lib` is textually last —
    // `build_graph`'s LIFO crawl (`queue.pop()`) explores the *last*-pushed
    // import first, so `other-lib` (and therefore its nested `pkg`
    // duplicate) is discovered — and assigned a module id — before the
    // hoisted `pkg/index.js` shim, reproducing the exact discovery-order
    // shape that exposes the `by_specifier` first-writer-wins collision.
    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import HostedPkg from 'pkg';
import consumer1Marker from 'consumer1';
import consumer2Marker from 'consumer2';
import consumer3Marker from 'consumer3';
import consumer4Marker from 'consumer4';
import consumer5Marker from 'consumer5';
import otherLibResult from 'other-lib';

document.getElementById('root').innerHTML =
  '<div id="output">' +
  [
    HostedPkg.MARKER,
    otherLibResult.otherLibMarker,
    consumer1Marker,
    consumer2Marker,
    consumer3Marker,
    consumer4Marker,
    consumer5Marker,
  ].join('|') +
  '</div>';
"#,
    )
    .expect("write entry");
}

/// #2267 round 2 — a package's only public surface may be a **classic
/// 2-line CJS conditional entry shim** (`if (process.env.NODE_ENV ===
/// 'production') module.exports = require('./cjs/pkg.production.js') else
/// ...`, zero ESM export statements) hoisted to the top-level
/// `node_modules/`, *and* the graph may separately carry a non-hoisted,
/// nested duplicate of the exact same package under some other package's
/// own `node_modules/` (a realistic version-conflict shape — this is
/// exactly the react/react-is shape a real `@mui` corpus hit: module 1133
/// = `node_modules/react/index.js`, referenced from ~180 files, never
/// emitted). Both copies textually reduce to the identical bare specifier
/// via `tree_shake::package_specifier_variants` (it only looks at the text
/// after the *last* `node_modules/` segment), and
/// `tree_shake::ModuleLookup`'s `by_specifier` fast path used to let
/// whichever copy was discovered first during the graph crawl silently
/// win specifier resolution for *every* consumer in the whole graph —
/// including consumers whose real, importer-aware resolution should have
/// landed on the *other* copy. That starved the correctly-hoisted copy of
/// all analysis-side `used_exports` demand despite the real graph's own
/// per-importer resolution consistently pointing every external consumer
/// at it, so `compute_transform_survivors` dropped it from the survivors
/// set while compiled consumer code still referenced its numeric module id
/// — caught loudly by `Bundler::check_referenced_module_emission` rather
/// than silently producing a black page.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn entry_flatten_resolves_hoisted_cjs_entry_shim_despite_nested_duplicate_specifier_collision(
) -> Result<()> {
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_cjs_entry_shim_specifier_collision_fixture(fixture);

    // The build itself must succeed: pre-fix, the hoisted `pkg` shim is
    // dropped from the survivors set while every external consumer's
    // compiled glue still calls into it by numeric id, and
    // `check_referenced_module_emission` turns that into a loud build
    // failure rather than a silent black page. `--no-minify` keeps this
    // test isolated from a separate, pre-existing minifier issue (default
    // minification can strip property-mutation-only CJS leaf bodies like
    // `exports.MARKER = ...` down to an empty object even when they are
    // correctly marked fully-live — unrelated to this fix; see the report
    // follow-ups) so this test's signal is specifically the specifier
    // resolution / survivors-set behavior this fix targets.
    require_success(
        run_jet(fixture, ["build", "--no-minify"])?,
        "build (CJS entry-shim specifier-collision fixture)",
    )?;
    assert!(
        fixture.join("dist/index.html").exists(),
        "build must emit dist/index.html"
    );

    let dist = fixture.join("dist");

    // (a) Static guard (module presence, cheap, no browser required): both
    // the hoisted and nested `pkg` bodies must have been emitted, not just
    // referenced.
    let hoisted_emitted = dist_js_files_containing(&dist, "HOISTED_PKG");
    assert!(
        !hoisted_emitted.is_empty(),
        "expected the hoisted pkg shim's body ('HOISTED_PKG') to be \
         emitted somewhere in dist — it must survive tree-shaking's \
         analysis pre-pass despite the nested duplicate's specifier \
         collision, not just be referenced by numeric id from every \
         consumer's compiled glue"
    );
    let nested_emitted = dist_js_files_containing(&dist, "NESTED_PKG");
    assert!(
        !nested_emitted.is_empty(),
        "expected other-lib's own nested pkg duplicate ('NESTED_PKG') to \
         also survive — the fix must not break legitimate nearest-wins \
         resolution for the one consumer that really should get the \
         nested copy"
    );

    // (b) Dynamic proof: the app must actually boot in a real Chromium and
    // every one of the 5 external fan-in consumers plus the entry's own
    // direct import must render the *hoisted* marker, while `other-lib`'s
    // own import (from inside its own nested tree) renders the *nested*
    // marker — proving the analysis's resolution matches the real graph's
    // resolution for both sides of the collision, not merely that
    // something got emitted.
    if !common::chromium_available() {
        eprintln!(
            "skipping real-browser half of \
             entry_flatten_resolves_hoisted_cjs_entry_shim_despite_nested_duplicate_specifier_collision: \
             no Chromium available"
        );
        return Ok(());
    }

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

        shutdown_browser(fixture, &mut browser).context("jet browser shutdown")?;
        Ok::<Option<String>, anyhow::Error>(output_text)
    })
    .await;

    let output_text = match result {
        Ok(inner) => inner?,
        Err(_) => {
            let _ = Command::new(env!("CARGO_BIN_EXE_jet"))
                .args(["browser", "shutdown"])
                .current_dir(fixture)
                .output();
            return Err(anyhow!(
                "entry_flatten_resolves_hoisted_cjs_entry_shim_despite_nested_duplicate_specifier_collision \
                 timed out after 60s"
            ));
        }
    };

    assert_eq!(
        output_text.as_deref(),
        Some("HOISTED_PKG|NESTED_PKG|HOISTED_PKG|HOISTED_PKG|HOISTED_PKG|HOISTED_PKG|HOISTED_PKG"),
        "app must boot with the entry's own direct import and every \
         external fan-in consumer resolving to the hoisted pkg copy, \
         while other-lib's own import (from inside its own nested tree) \
         keeps resolving to its nested duplicate (#2267 round 2)"
    );

    Ok(())
}
// HANDWRITE-END
