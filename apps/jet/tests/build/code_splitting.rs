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

use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
// HANDWRITE-END
