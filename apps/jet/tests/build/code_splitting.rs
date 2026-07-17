// HANDWRITE-BEGIN gap="missing-generator:unit-test:9bd20147" tracker="pending-tracker" reason="Integration tests: `jet build --splitting` emits per-chunk asset files driven by a runtime chunk manifest (entry excludes lazy-loaded source, async chunk files self-register via __jet__.registerChunk, and the manifest maps chunk names/module ids to their final content-hashed filenames); `jet build` without the flag stays single-file with the pre-#1930 Promise.resolve(require(id)) dynamic-import lowering and carries no dynamicImport/chunkManifest/registerChunk tokens; and `--splitting` no longer emits the retired GH #3705 no-op warning."
//! `jet build --splitting` chunk codegen + runtime loader integration
//! coverage (WI #1930).
//!
//! The fixture is authored directly into a `tempfile::tempdir()` (no
//! checked-in `tests/fixtures/` dir needed): a plain-JS entry that
//! statically imports one helper module (inlined into the entry either way
//! — single-reference modules are not promoted to their own chunk) and
//! dynamically `import()`s two sibling modules, each carrying a distinct
//! marker string so chunk membership is verifiable by substring/manifest
//! inspection instead of parsing minified JS output.

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

/// Writes the shared code-splitting fixture (entry + statically-imported
/// helper + 2 dynamically-imported modules) under `dir/src/`.
fn write_splitting_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import { shared } from './shared.js';

console.log('ENTRY_MARKER', shared());

import('./lazy1.js').then((mod) => mod.default());
import('./lazy2.js').then((mod) => mod.default());
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("src/shared.js"),
        "export function shared() { return 'SHARED_MARKER'; }\n",
    )
    .expect("write shared");
    fs::write(
        dir.join("src/lazy1.js"),
        "export default function lazy1() { return 'LAZY_ONE_MARKER'; }\n",
    )
    .expect("write lazy1");
    fs::write(
        dir.join("src/lazy2.js"),
        "export default function lazy2() { return 'LAZY_TWO_MARKER'; }\n",
    )
    .expect("write lazy2");
}

/// Recursively collects every regular file under `dir`.
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

/// Extracts the JSON value assigned to `<marker> = {...}` inside `code`,
/// tolerant of minified spacing (`marker=` vs `marker = `) — parses only
/// the leading JSON value after the first `=` following `marker` and
/// ignores whatever comes after it (`;`, more statements).
///
/// Uses the *last* occurrence of `marker`, not the first: the generated
/// runtime's `loadChunk`/`dynamicImport` helpers read
/// `window.__jet__.chunkManifest` (with a fallback default) earlier in the
/// entry file, and only the final occurrence is the real `marker = {...}`
/// assignment this helper wants to parse.
fn extract_json_assignment(code: &str, marker: &str) -> Result<Value> {
    let marker_idx = code
        .rfind(marker)
        .with_context(|| format!("code missing {marker:?}"))?;
    let after_marker = &code[marker_idx + marker.len()..];
    let eq_rel = after_marker
        .find('=')
        .with_context(|| format!("no '=' found after {marker:?}"))?;
    let tail = &after_marker[eq_rel + 1..];
    serde_json::Deserializer::from_str(tail)
        .into_iter::<Value>()
        .next()
        .transpose()
        .with_context(|| format!("invalid JSON after {marker:?} ="))?
        .with_context(|| format!("no JSON value found after {marker:?} ="))
}

#[test]
fn splitting_flag_emits_chunk_files_with_manifest_and_registered_async_chunks() -> Result<()> {
    // `--no-minify`: `build_chunk_manifest_js` emits the manifest via
    // `serde_json::to_string` (strict, double-quoted JSON), but the default
    // minify pass then rewrites that same object literal's string-quote
    // style along with the rest of the entry bundle (e.g. plain string
    // values become backtick literals). Keeping minify off makes the
    // manifest exactly the JSON `build_chunk_manifest_js` produced, so
    // `extract_json_assignment` below can parse it with `serde_json`
    // instead of hand-rolling a JS-object-literal parser.
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_splitting_fixture(fixture);

    require_success(
        run_jet(fixture, ["build", "--splitting", "--no-minify"])?,
        "build --splitting --no-minify",
    )?;

    let dist = fixture.join("dist");
    let files = list_files_recursive(&dist);
    assert!(
        files.len() >= 3,
        "splitting build must emit >=3 files (entry + >=2 chunks), got {files:?}"
    );

    let entry_path = files
        .iter()
        .find(|p| {
            p.parent() == Some(dist.as_path())
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("main.") && n.ends_with(".js"))
        })
        .unwrap_or_else(|| panic!("no top-level main.<hash>.js entry file among {files:?}"));
    let entry_code = fs::read_to_string(entry_path).expect("read entry file");

    assert!(
        !entry_code.contains("LAZY_ONE_MARKER") && !entry_code.contains("LAZY_TWO_MARKER"),
        "entry chunk must exclude lazy-loaded source; entry={}",
        entry_path.display()
    );
    assert!(
        entry_code.contains("ENTRY_MARKER") && entry_code.contains("SHARED_MARKER"),
        "entry chunk must still contain its own source plus the statically-imported helper"
    );
    assert!(
        entry_code.contains("__jet__.dynamicImport("),
        "entry must lower import() to the chunk-aware runtime loader"
    );
    assert!(
        entry_code.contains("__jet__.require("),
        "entry must still boot itself via the runtime require() call"
    );

    let manifest = extract_json_assignment(&entry_code, "__jet__.chunkManifest")
        .expect("entry must carry a parseable __jet__.chunkManifest assignment");
    let chunks_obj = manifest
        .get("chunks")
        .and_then(Value::as_object)
        .expect("manifest.chunks must be an object");
    let module_chunks_obj = manifest
        .get("moduleChunks")
        .and_then(Value::as_object)
        .expect("manifest.moduleChunks must be an object");

    for chunk_name in ["chunk-lazy1", "chunk-lazy2"] {
        let manifest_entry = chunks_obj
            .get(chunk_name)
            .unwrap_or_else(|| panic!("manifest.chunks missing {chunk_name:?}: {chunks_obj:?}"));
        let manifest_file = manifest_entry
            .get("file")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("manifest.chunks[{chunk_name:?}].file missing/non-string"));
        assert!(
            manifest_file.starts_with("assets/") && manifest_file.ends_with(".js"),
            "chunk file {manifest_file:?} must live under assets/ as a .js file"
        );

        let chunk_disk_path = dist.join(manifest_file);
        assert!(
            chunk_disk_path.exists(),
            "manifest names {manifest_file:?} for {chunk_name:?} but no such file was written"
        );

        let chunk_code = fs::read_to_string(&chunk_disk_path)
            .with_context(|| format!("read chunk file {}", chunk_disk_path.display()))?;
        assert!(
            chunk_code.contains(&format!("__jet__.registerChunk(\"{chunk_name}\"")),
            "chunk file for {chunk_name:?} must self-register under its own name; code={chunk_code}"
        );
    }

    assert!(
        dist.join("assets").is_dir(),
        "splitting build must write chunk files under dist/assets/"
    );
    assert!(
        module_chunks_obj
            .values()
            .any(|v| v.as_str() == Some("chunk-lazy1")),
        "moduleChunks must map at least one module id to chunk-lazy1: {module_chunks_obj:?}"
    );
    assert!(
        module_chunks_obj
            .values()
            .any(|v| v.as_str() == Some("chunk-lazy2")),
        "moduleChunks must map at least one module id to chunk-lazy2: {module_chunks_obj:?}"
    );

    Ok(())
}

#[test]
fn without_splitting_flag_build_stays_single_file_with_legacy_dynamic_import_lowering() -> Result<()>
{
    // Default-flags build (minify on, matching a real `jet build` run):
    // checks the emitted-artifact *shape* — single file, no chunk
    // directory, none of the new splitting-only runtime tokens leak in.
    // String literals (the LAZY_*_MARKER exports) survive minification
    // even though local identifiers get mangled, so those stay checkable
    // here too.
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_splitting_fixture(fixture);

    require_success(run_jet(fixture, ["build"])?, "build (no --splitting)")?;

    let dist = fixture.join("dist");
    assert!(
        !dist.join("assets").exists(),
        "build without --splitting must not create a dist/assets/ chunk directory"
    );

    let js_files: Vec<PathBuf> = list_files_recursive(&dist)
        .into_iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("js"))
        .collect();
    assert_eq!(
        js_files.len(),
        1,
        "build without --splitting must emit exactly one JS bundle, got {js_files:?}"
    );
    let entry_path = &js_files[0];
    assert!(
        entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("main.") && n.ends_with(".js")),
        "the single JS file must be the main.<hash>.js entry, got {entry_path:?}"
    );

    let code = fs::read_to_string(entry_path).expect("read entry file");
    assert!(
        !code.contains("__jet__.dynamicImport("),
        "OFF path must not use the chunk-aware dynamicImport lowering"
    );
    assert!(
        !code.contains("__jet__.chunkManifest"),
        "OFF path must not inject a chunk manifest"
    );
    assert!(
        !code.contains("registerChunk"),
        "OFF path must not wrap any code in registerChunk"
    );
    // Splitting off still bundles every reachable module (including
    // dynamic-import targets) into the one file — AC2 byte-stability is
    // about the emitted-artifact shape, not about dynamic imports
    // stopping to resolve.
    assert!(
        code.contains("LAZY_ONE_MARKER") && code.contains("LAZY_TWO_MARKER"),
        "OFF path must still inline dynamically-imported module bodies into the single bundle"
    );

    // Second build, `--no-minify`, isolated in its own tempdir: verifies
    // the *exact* pre-#1930 dynamic-import lowering shape
    // (`Promise.resolve(require(id))`). The default build above can't
    // assert this literal shape — minification is free to mangle the
    // scope-hoisted bundle's local `require` parameter (observed: renamed
    // to a single short identifier), which is pre-existing minifier
    // behavior unrelated to #1930, not a splitting concern.
    let temp_raw = tempfile::tempdir().context("tempdir (no-minify)")?;
    let fixture_raw = temp_raw.path();
    write_splitting_fixture(fixture_raw);
    require_success(
        run_jet(fixture_raw, ["build", "--no-minify"])?,
        "build --no-minify (no --splitting)",
    )?;
    let raw_dist = fixture_raw.join("dist");
    let raw_js_files: Vec<PathBuf> = list_files_recursive(&raw_dist)
        .into_iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("js"))
        .collect();
    assert_eq!(
        raw_js_files.len(),
        1,
        "build --no-minify without --splitting must emit exactly one JS bundle, got {raw_js_files:?}"
    );
    let raw_code = fs::read_to_string(&raw_js_files[0]).expect("read entry file (no-minify)");
    assert!(
        raw_code.contains("Promise.resolve(require("),
        "OFF path must keep the pre-#1930 Promise.resolve(require(id)) dynamic-import lowering; code={raw_code}"
    );

    Ok(())
}

#[test]
fn splitting_flag_no_longer_warns() -> Result<()> {
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_splitting_fixture(fixture);

    let output = require_success(
        run_jet(fixture, ["build", "--splitting"])?,
        "build --splitting",
    )?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("GH #3705"),
        "the retired GH #3705 no-op warning must not fire once --splitting is wired: stderr={stderr}"
    );
    assert!(
        !stderr.contains("--splitting is currently a no-op"),
        "the retired GH #3705 no-op warning text must not fire: stderr={stderr}"
    );

    Ok(())
}

// ── WI #1931: per-chunk sourcemaps (AC1) ────────────────────────────────────

#[test]
fn splitting_external_sourcemap_emits_per_chunk_map_files_with_bare_basename_url() -> Result<()> {
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_splitting_fixture(fixture);

    require_success(
        run_jet(fixture, ["build", "--splitting", "--sourcemap", "external"])?,
        "build --splitting --sourcemap external",
    )?;

    let dist = fixture.join("dist");
    let chunk_js_files: Vec<PathBuf> = list_files_recursive(&dist.join("assets"))
        .into_iter()
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()) == Some("js")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("chunk-"))
        })
        .collect();
    assert_eq!(
        chunk_js_files.len(),
        2,
        "expected exactly 2 async chunk JS files, got {chunk_js_files:?}"
    );

    for chunk_js in &chunk_js_files {
        let code =
            fs::read_to_string(chunk_js).with_context(|| format!("read {}", chunk_js.display()))?;
        let basename = chunk_js
            .file_name()
            .and_then(|n| n.to_str())
            .expect("chunk file has a name");

        let bare_comment = format!("//# sourceMappingURL={basename}.map");
        assert!(
            code.contains(&bare_comment),
            "chunk {basename} must carry a bare-basename sourceMappingURL comment; code tail={:?}",
            &code[code.len().saturating_sub(200)..]
        );
        assert!(
            !code.contains(&format!("sourceMappingURL=assets/{basename}.map")),
            "chunk {basename} sourceMappingURL must not double-prefix assets/ \
             (browsers resolve it relative to the chunk file's own directory)"
        );

        let map_path = chunk_js.with_extension("js.map");
        assert!(
            map_path.exists(),
            "expected sibling map file {}",
            map_path.display()
        );
        let map_json: Value = serde_json::from_str(&fs::read_to_string(&map_path)?)
            .with_context(|| format!("parse {} as JSON", map_path.display()))?;
        assert_eq!(
            map_json.get("file").and_then(Value::as_str),
            Some(basename),
            "map file's \"file\" field must match the chunk's bare basename"
        );
    }

    Ok(())
}

#[test]
fn splitting_sourcemap_modes_inline_hidden_none_match_flag_semantics() -> Result<()> {
    fn chunk_js_files(dist: &Path) -> Vec<PathBuf> {
        list_files_recursive(&dist.join("assets"))
            .into_iter()
            .filter(|p| {
                p.extension().and_then(|e| e.to_str()) == Some("js")
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with("chunk-"))
            })
            .collect()
    }
    fn no_map_files(dist: &Path) -> bool {
        list_files_recursive(dist)
            .iter()
            .all(|p| p.extension().and_then(|e| e.to_str()) != Some("map"))
    }

    // Inline: no separate .map files anywhere; base64 data URL embedded in
    // each chunk's own code.
    {
        let temp = tempfile::tempdir().context("tempdir (inline)")?;
        let fixture = temp.path();
        write_splitting_fixture(fixture);
        require_success(
            run_jet(fixture, ["build", "--splitting", "--sourcemap", "inline"])?,
            "build --splitting --sourcemap inline",
        )?;
        let dist = fixture.join("dist");
        assert!(
            no_map_files(&dist),
            "inline mode must not write any .map files"
        );
        let chunks = chunk_js_files(&dist);
        assert_eq!(
            chunks.len(),
            2,
            "expected 2 async chunk files, got {chunks:?}"
        );
        for chunk_js in &chunks {
            let code = fs::read_to_string(chunk_js)?;
            assert!(
                code.contains("sourceMappingURL=data:application/json;base64,"),
                "inline mode chunk must carry a base64 data-URL source map: {}",
                chunk_js.display()
            );
        }
    }

    // Hidden: .map file IS written, but the chunk code omits the
    // sourceMappingURL comment (mirrors bundler/lib_build.rs's
    // apply_library_sourcemap reference Hidden-mode behavior).
    {
        let temp = tempfile::tempdir().context("tempdir (hidden)")?;
        let fixture = temp.path();
        write_splitting_fixture(fixture);
        require_success(
            run_jet(fixture, ["build", "--splitting", "--sourcemap", "hidden"])?,
            "build --splitting --sourcemap hidden",
        )?;
        let dist = fixture.join("dist");
        let chunks = chunk_js_files(&dist);
        assert_eq!(
            chunks.len(),
            2,
            "expected 2 async chunk files, got {chunks:?}"
        );
        for chunk_js in &chunks {
            let map_path = chunk_js.with_extension("js.map");
            assert!(
                map_path.exists(),
                "hidden mode must still write {}",
                map_path.display()
            );
            let code = fs::read_to_string(chunk_js)?;
            assert!(
                !code.contains("sourceMappingURL"),
                "hidden mode chunk must omit the sourceMappingURL comment: {}",
                chunk_js.display()
            );
        }
    }

    // None: no map artifacts and no sourceMappingURL comment anywhere.
    {
        let temp = tempfile::tempdir().context("tempdir (none)")?;
        let fixture = temp.path();
        write_splitting_fixture(fixture);
        require_success(
            run_jet(fixture, ["build", "--splitting", "--sourcemap", "none"])?,
            "build --splitting --sourcemap none",
        )?;
        let dist = fixture.join("dist");
        assert!(
            no_map_files(&dist),
            "none mode must not write any .map files"
        );
        let chunks = chunk_js_files(&dist);
        assert_eq!(
            chunks.len(),
            2,
            "expected 2 async chunk files, got {chunks:?}"
        );
        for chunk_js in &chunks {
            let code = fs::read_to_string(chunk_js)?;
            assert!(
                !code.contains("sourceMappingURL"),
                "none mode chunk must not carry a sourceMappingURL comment: {}",
                chunk_js.display()
            );
        }
    }

    Ok(())
}

// ── WI #1931: preload hints in index.html (AC2) ─────────────────────────────

/// Writes a fixture shaped like `bundler::splitting`'s own
/// "diamond dynamic boundary" coverage (`test_diamond_dynamic_boundary_shared`
/// / `test_preload_hints_multi_chunk`): the entry AND one of the two lazy
/// chunks both statically import `common.js`, so it crosses the 2+-chunk
/// reachability threshold and gets promoted to a real `"shared"` chunk —
/// unlike `write_splitting_fixture`'s `shared.js`, which only the entry ever
/// imports and therefore never promotes (kept that way deliberately so the
/// existing manifest tests stay a minimal 2-async-chunk fixture).
fn write_shared_promotion_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import { common } from './common.js';

console.log('ENTRY_MARKER', common());

import('./lazy1.js').then((mod) => mod.default());
import('./lazy2.js').then((mod) => mod.default());
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("src/common.js"),
        "export function common() { return 'COMMON_MARKER'; }\n",
    )
    .expect("write common");
    fs::write(
        dir.join("src/lazy1.js"),
        r#"import { common } from './common.js';

export default function lazy1() { return 'LAZY_ONE_MARKER:' + common(); }
"#,
    )
    .expect("write lazy1");
    fs::write(
        dir.join("src/lazy2.js"),
        "export default function lazy2() { return 'LAZY_TWO_MARKER'; }\n",
    )
    .expect("write lazy2");
}

#[test]
fn splitting_preload_hints_cover_shared_chunk_and_exclude_async_chunks_in_index_html() -> Result<()>
{
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_shared_promotion_fixture(fixture);

    require_success(
        run_jet(fixture, ["build", "--splitting"])?,
        "build --splitting",
    )?;

    let dist = fixture.join("dist");
    let shared_js = list_files_recursive(&dist.join("assets"))
        .into_iter()
        .find(|p| {
            p.extension().and_then(|e| e.to_str()) == Some("js")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("shared."))
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a promoted shared.<hash>.js chunk under {}",
                dist.join("assets").display()
            )
        });
    let shared_basename = shared_js
        .file_name()
        .and_then(|n| n.to_str())
        .expect("shared chunk file has a name")
        .to_string();

    let html = fs::read_to_string(dist.join("index.html")).context("read dist/index.html")?;

    let expected_tag =
        format!(r#"<link rel="preload" as="script" href="assets/{shared_basename}">"#);
    assert!(
        html.contains(&expected_tag),
        "index.html must preload the shared chunk with a classic-script hint: \
         expected {expected_tag:?}; html={html}"
    );
    assert!(
        !html.contains("chunk-lazy1") && !html.contains("chunk-lazy2"),
        "index.html must not preload either async chunk: html={html}"
    );
    assert!(
        !html.contains("modulepreload"),
        "code-split chunks load via classic <script> injection, not ESM import; \
         index.html must not use rel=\"modulepreload\": html={html}"
    );

    Ok(())
}

// ── WI #1931: parallel-tail wall-time sanity (scope item 5) ────────────────

/// Log-only wall-time comparison between the parallel per-chunk minify tail
/// (default) and the serial fallback path (`JET_MINIFY_STAGE_DUMP` set —
/// see the `hashed_chunks` construction in `cli.rs`).
///
/// Not an assertion on timing: `write_splitting_fixture` only has 2 tiny
/// chunks, so real per-chunk work is a handful of milliseconds — well
/// inside the noise floor of process spawn + rayon thread-pool-first-use
/// overhead on a shared dev/CI machine. A strict `parallel <= serial * K`
/// assertion on a fixture this small would be a coin flip more often than a
/// real regression signal, so this logs both durations and only asserts the
/// correctness invariant that matters: both paths must still produce the
/// same chunk set.
#[test]
fn splitting_parallel_chunk_tail_wall_time_sanity() -> Result<()> {
    fn chunk_names(dist: &Path) -> Vec<String> {
        let mut names: Vec<String> = list_files_recursive(&dist.join("assets"))
            .into_iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string))
            .filter(|n| n.starts_with("chunk-") && n.ends_with(".js"))
            .map(|n| n.split('.').next().unwrap_or_default().to_string())
            .collect();
        names.sort();
        names.dedup();
        names
    }

    let temp_parallel = tempfile::tempdir().context("tempdir (parallel)")?;
    let fixture_parallel = temp_parallel.path();
    write_splitting_fixture(fixture_parallel);
    let start_parallel = std::time::Instant::now();
    require_success(
        run_jet(fixture_parallel, ["build", "--splitting"])?,
        "build --splitting (parallel tail)",
    )?;
    let parallel_duration = start_parallel.elapsed();

    let temp_serial = tempfile::tempdir().context("tempdir (serial)")?;
    let fixture_serial = temp_serial.path();
    write_splitting_fixture(fixture_serial);
    let dump_dir = fixture_serial.join(".stage-dump");
    let start_serial = std::time::Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_jet"))
        .args(["build", "--splitting"])
        .current_dir(fixture_serial)
        .env("JET_MINIFY_STAGE_DUMP", &dump_dir)
        .output()
        .context("run jet build --splitting (serial tail via JET_MINIFY_STAGE_DUMP)")?;
    let serial_duration = start_serial.elapsed();
    require_success(
        output,
        "build --splitting (serial tail via JET_MINIFY_STAGE_DUMP)",
    )?;

    eprintln!(
        "[wall-time] splitting chunk tail: parallel={parallel_duration:?} serial={serial_duration:?}"
    );

    assert_eq!(
        chunk_names(&fixture_parallel.join("dist")),
        chunk_names(&fixture_serial.join("dist")),
        "parallel and serial chunk tails must produce the same chunk set"
    );

    Ok(())
}

// ── WI #1931: real-browser lazy-load smoke (AC3) ────────────────────────────
//
// Serves a `--splitting` build's `dist/` over local HTTP and drives real
// Chromium through the `jet browser launch/eval/shutdown` CLI (a session
// file at `.jet/browser-session.json`, not an in-process CDP client) —
// the exact pattern already proven in
// `tests/build/production_build_regression.rs::production_build_regression_fixture_boots_in_browser`,
// duplicated here (rather than extracted into a shared helper) since that
// file is otherwise unrelated to code splitting and extraction would be its
// own out-of-scope refactor. Skips (does not fail) when Chromium isn't
// available, matching `tests/browser-bridge/page_api_parity.rs`'s
// skip-and-return pattern so CI without Chromium stays green.

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

/// Writes a fixture where clicking a button triggers a dynamic `import()`
/// for the first time, so a real browser can prove the chunk file is not
/// requested until the click happens. Mounts into the default HTML
/// template's `#root` div (`frontend::default_index_html`).
fn write_lazy_click_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"document.getElementById('root').innerHTML =
  '<button id="load-btn">Load</button><div id="output"></div>';

document.getElementById('load-btn').addEventListener('click', () => {
  import('./lazy.js').then((mod) => {
    document.getElementById('output').textContent = mod.default();
  });
});
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("src/lazy.js"),
        "export default function lazy() { return 'LAZY_LOADED_MARKER'; }\n",
    )
    .expect("write lazy");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn splitting_lazy_chunk_loads_on_demand_in_real_browser() -> Result<()> {
    if !common::chromium_available() {
        eprintln!(
            "skipping splitting_lazy_chunk_loads_on_demand_in_real_browser: no Chromium available"
        );
        return Ok(());
    }

    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_lazy_click_fixture(fixture);

    require_success(
        run_jet(fixture, ["build", "--splitting"])?,
        "build --splitting",
    )?;
    assert!(
        fixture.join("dist/index.html").exists(),
        "splitting build must emit dist/index.html"
    );

    // Hard bound so a stuck CDP round-trip fails the test instead of
    // hanging the run (WI #1931's explicit anti-hang requirement). Every
    // inner wait loop below is already independently bounded (session-file
    // poll <=15s, each readiness poll <=12s); this is defense in depth
    // around the whole sequence, generous enough for a cold Chromium
    // launch.
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

        let mut booted = false;
        for _ in 0..120 {
            let ready = browser_eval_json(fixture, "document.getElementById('load-btn') !== null")
                .unwrap_or(Value::Bool(false));
            if ready.as_bool() == Some(true) {
                booted = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        assert!(booted, "fixture page did not boot (no #load-btn found)");

        let before = browser_eval_json(
            fixture,
            "performance.getEntriesByType('resource').map((e) => e.name)",
        )
        .context("read pre-click resource timing")?;
        let before_names: Vec<String> = before
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        assert!(
            !before_names.iter().any(|n| n.contains("chunk-lazy")),
            "lazy chunk must not be requested before the click: {before_names:?}"
        );

        // Deterministic trigger via the bridge: run the click itself as a
        // page-context JS expression rather than a synthetic CDP mouse
        // event — no coordinate/layout dependency.
        browser_eval_json(
            fixture,
            "(() => { document.getElementById('load-btn').click(); return true; })()",
        )
        .context("click #load-btn")?;

        let mut output_text = None;
        for _ in 0..120 {
            let value = browser_eval_json(fixture, "document.getElementById('output').textContent")
                .unwrap_or(Value::Null);
            if let Some(text) = value.as_str() {
                if text == "LAZY_LOADED_MARKER" {
                    output_text = Some(text.to_string());
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        assert_eq!(
            output_text.as_deref(),
            Some("LAZY_LOADED_MARKER"),
            "clicking #load-btn must render the lazily-imported module's output"
        );

        let after = browser_eval_json(
            fixture,
            "performance.getEntriesByType('resource').map((e) => e.name)",
        )
        .context("read post-click resource timing")?;
        let after_names: Vec<String> = after
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        assert!(
            after_names.iter().any(|n| n.contains("chunk-lazy")),
            "lazy chunk must be requested after the click: {after_names:?}"
        );

        shutdown_browser(fixture, &mut browser).context("jet browser shutdown")?;
        Ok::<(), anyhow::Error>(())
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => {
            // Best-effort cleanup: the spawned `jet browser launch` child
            // is otherwise orphaned once the timeout drops this future.
            let _ = Command::new(env!("CARGO_BIN_EXE_jet"))
                .args(["browser", "shutdown"])
                .current_dir(fixture)
                .output();
            Err(anyhow!(
                "splitting_lazy_chunk_loads_on_demand_in_real_browser timed out after 60s"
            ))
        }
    }
}
// HANDWRITE-END
