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

/// Normalizes an entry file's code for cross-build byte-identity
/// comparison: blanks the trailing `//# sourceMappingURL=...` comment
/// (whose filename embeds the entry's own content hash) and splits the
/// `__jet__.chunkManifest = {...}` assignment out into a separately
/// returned, parsed `Value` (with the code's own copy of that JSON blob
/// replaced by a fixed placeholder).
///
/// Needed because `build_chunk_manifest_js`'s `chunks`/`moduleChunks` key
/// order is not currently stable run-to-run for otherwise byte-identical
/// input — confirmed via 3 back-to-back `--splitting` builds of the exact
/// same fixture with the survivor filter in the *same* state (ON) in all
/// three, producing 2 distinct manifest key orderings and thus 2 distinct
/// entry content hashes. That non-determinism sits entirely upstream of
/// this WI (traced as far as `build_chunk_manifest_js` in `src/cli.rs`,
/// whose input chunk-slice order is not itself sorted) and is reported
/// separately rather than fixed here; this helper makes the byte-identity
/// tests below robust to it while still proving genuine byte-identity for
/// everything the survivor filter can actually affect.
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
    // `--no-splitting`: since WI #1932, a web build with dynamic imports
    // splits *by default* — this test exercises the escape hatch
    // explicitly instead of relying on flag-less defaults (see the
    // `wi_1932_default_on_splitting` tests below for default-on coverage).
    // Default-flags build (minify on, matching a real `jet build
    // --no-splitting` run): checks the emitted-artifact *shape* — single
    // file, no chunk directory, none of the new splitting-only runtime
    // tokens leak in. String literals (the LAZY_*_MARKER exports) survive
    // minification even though local identifiers get mangled, so those
    // stay checkable here too.
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_splitting_fixture(fixture);

    require_success(
        run_jet(fixture, ["build", "--no-splitting"])?,
        "build --no-splitting",
    )?;

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
        run_jet(fixture_raw, ["build", "--no-minify", "--no-splitting"])?,
        "build --no-minify --no-splitting",
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

// ── #1948: [build.manual_chunks] config plumbing ────────────────────────────
//
// `write_manual_chunks_fixture` writes a real npm-style `node_modules/`
// package directory (bare-specifier-resolvable with no `jet install` step —
// same technique as `write_symlink_and_alias_fixture`'s `shared-dep`, minus
// the pnpm-symlink indirection this doesn't need) statically imported once
// by the entry, plus two unrelated dynamic imports. It does NOT write a
// `jet.toml` itself, so the exact same source proves both "configured" (AC1)
// and "absent" (AC2) from one fixture.

/// Writes `node_modules/fakepkg/{package.json,index.js}` plus a `src/` entry
/// that statically imports it (single-reference, so nothing but explicit
/// `manual_chunks` routing could ever pull it out of the entry chunk) and
/// dynamically imports two sibling modules, to prove manual-chunk routing
/// leaves unrelated async chunks untouched.
fn write_manual_chunks_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("node_modules/fakepkg")).expect("create node_modules/fakepkg");
    fs::write(
        dir.join("node_modules/fakepkg/package.json"),
        r#"{"name":"fakepkg","version":"1.0.0","main":"index.js"}"#,
    )
    .expect("write fakepkg package.json");
    fs::write(
        dir.join("node_modules/fakepkg/index.js"),
        "export function fakepkg() { return 'FAKEPKG_MARKER'; }\n",
    )
    .expect("write fakepkg index.js");

    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import { fakepkg } from 'fakepkg';

console.log('ENTRY_MARKER', fakepkg());

import('./lazy1.js').then((mod) => mod.default());
import('./lazy2.js').then((mod) => mod.default());
"#,
    )
    .expect("write entry");
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

/// Locates the single top-level `main.<hash>.js` entry file under `dist/`.
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

/// #1948 AC1 — `[build.manual_chunks]` in `jet.toml` routes a glob-matched
/// module (a statically-imported, single-reference npm package — the exact
/// vendor-splitting shape from the issue) into its own named
/// `ChunkType::Shared` chunk: excluded from the entry, carried by the named
/// chunk file, earning a preload hint (via the entry-imports wiring added
/// alongside this config plumbing), and leaving the unrelated lazy chunks
/// untouched.
///
/// Deliberately does NOT drive this fixture through a real-browser
/// `jet browser launch/eval` round-trip the way
/// `splitting_lazy_chunk_loads_on_demand_in_real_browser` does for plain
/// async chunks. Doing so during development of this change surfaced a
/// pre-existing runtime-loader gap (not introduced here — see the section
/// header above this test): a `ChunkType::Shared` chunk that is a *static*
/// dependency of the entry throws `Uncaught Error: Module not found: <id>`
/// in a real page, because the entry bootstrap ends in a bare synchronous
/// `__jet__.require(entry.id)` (see the `__jet__.chunkManifest = ...;
/// __jet__.require(0);` tail `Bundler::generate_split_bundle` /
/// `cli.rs::inject_chunk_manifest` emit) and nothing ever calls
/// `__jet__.loadChunk` for a shared/manual chunk ahead of that — the only
/// reference emitted anywhere is the non-executing `<link rel="preload">`
/// hint this test itself asserts on below. This reproduces for the
/// already-shipped auto-detected `shared` chunk too (WI #1930/#1931), not
/// just manual chunks; it is the exact "manual chunks don't flow through the
/// #1931 emission path cleanly" seam issue #1948's own STOP clause
/// anticipates, and is out of scope for this change (see #1948's final
/// report for the proposed fix shape).
#[test]
fn splitting_manual_chunks_config_routes_glob_matched_module_into_named_chunk_excluded_from_entry(
) -> Result<()> {
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_manual_chunks_fixture(fixture);
    fs::write(
        fixture.join("jet.toml"),
        "[build.manual_chunks]\nvendor = [\"**/node_modules/fakepkg/**\"]\n",
    )
    .expect("write jet.toml");

    require_success(
        run_jet(fixture, ["build", "--splitting", "--no-minify"])?,
        "build --splitting --no-minify (manual_chunks fixture)",
    )?;

    let dist = fixture.join("dist");
    let assets = list_files_recursive(&dist.join("assets"));

    let vendor_js = assets
        .iter()
        .find(|p| {
            p.extension().and_then(|e| e.to_str()) == Some("js")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("vendor."))
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a vendor.<hash>.js manual chunk under {}: found {:?}",
                dist.join("assets").display(),
                assets
            )
        });
    let vendor_basename = vendor_js
        .file_name()
        .and_then(|n| n.to_str())
        .expect("vendor chunk file has a name")
        .to_string();
    let vendor_code = fs::read_to_string(vendor_js).context("read vendor chunk")?;
    assert!(
        vendor_code.contains("FAKEPKG_MARKER"),
        "vendor chunk must carry the routed module's code: {vendor_code}"
    );

    let entry_path = find_entry_file(&dist);
    let entry_code = fs::read_to_string(&entry_path).context("read entry file")?;
    assert!(
        entry_code.contains("ENTRY_MARKER"),
        "entry must still contain its own source: {entry_code}"
    );
    assert!(
        !entry_code.contains("FAKEPKG_MARKER"),
        "entry must exclude the manually-chunked module's code: {entry_code}"
    );

    // App-level (lazy) chunks are unaffected by manual-chunk routing.
    for (marker, prefix) in [
        ("LAZY_ONE_MARKER", "chunk-lazy1."),
        ("LAZY_TWO_MARKER", "chunk-lazy2."),
    ] {
        let lazy_js = assets
            .iter()
            .find(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(prefix))
            })
            .unwrap_or_else(|| panic!("expected a {prefix}<hash>.js chunk: found {assets:?}"));
        let lazy_code = fs::read_to_string(lazy_js).context("read lazy chunk")?;
        assert!(
            lazy_code.contains(marker),
            "{prefix}*.js must still carry its own marker unchanged: {lazy_code}"
        );
    }

    let html = fs::read_to_string(dist.join("index.html")).context("read dist/index.html")?;
    let expected_tag =
        format!(r#"<link rel="preload" as="script" href="assets/{vendor_basename}">"#);
    assert!(
        html.contains(&expected_tag),
        "index.html must preload the manual vendor chunk: expected {expected_tag:?}; html={html}"
    );

    Ok(())
}

/// #1948 AC2 — identical fixture, no `jet.toml` at all: behavior must be
/// identical to pre-#1948 `--splitting` output. The single-reference
/// `fakepkg` import stays inlined in the entry (nothing promotes it — it's
/// imported only once, by the entry itself) and no `vendor` chunk is ever
/// produced.
#[test]
fn splitting_manual_chunks_config_absent_leaves_entry_unchanged() -> Result<()> {
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_manual_chunks_fixture(fixture);
    // Deliberately no jet.toml.

    require_success(
        run_jet(fixture, ["build", "--splitting", "--no-minify"])?,
        "build --splitting --no-minify (no manual_chunks config)",
    )?;

    let dist = fixture.join("dist");
    let assets = list_files_recursive(&dist.join("assets"));
    assert!(
        !assets.iter().any(|p| p
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("vendor."))),
        "no manual_chunks config must mean no vendor chunk: found {assets:?}"
    );

    let entry_path = find_entry_file(&dist);
    let entry_code = fs::read_to_string(&entry_path).context("read entry file")?;
    assert!(
        entry_code.contains("FAKEPKG_MARKER"),
        "no config: fakepkg must stay inlined in the entry exactly like pre-#1948 \
         --splitting output: {entry_code}"
    );

    Ok(())
}

/// #1948 — GH #3300's invalid-glob warn-and-drop contract, reached starting
/// from `jet.toml` instead of calling `split_chunks_with_config` directly
/// (already covered at the unit level by
/// `bundler::splitting::manual_chunks_all_invalid_patterns_does_not_break_sibling_chunks`).
/// A syntactically-broken pattern must not fail the build or panic; it
/// degrades to the same output as AC2 (no config at all), since an
/// all-invalid-patterns chunk matches zero modules.
#[test]
fn splitting_manual_chunks_config_invalid_glob_does_not_fail_build() -> Result<()> {
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_manual_chunks_fixture(fixture);
    fs::write(
        fixture.join("jet.toml"),
        "[build.manual_chunks]\nvendor = [\"node_modules/fakepkg/{\"]\n",
    )
    .expect("write jet.toml with an unclosed-brace glob");

    require_success(
        run_jet(fixture, ["build", "--splitting", "--no-minify"])?,
        "build --splitting --no-minify (invalid manual_chunks glob must warn, not fail)",
    )?;

    let dist = fixture.join("dist");
    let assets = list_files_recursive(&dist.join("assets"));
    assert!(
        !assets.iter().any(|p| p
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("vendor."))),
        "an all-invalid-patterns chunk must match zero modules: found {assets:?}"
    );

    let entry_path = find_entry_file(&dist);
    let entry_code = fs::read_to_string(&entry_path).context("read entry file")?;
    assert!(
        entry_code.contains("FAKEPKG_MARKER"),
        "invalid glob must degrade gracefully, leaving fakepkg inlined: {entry_code}"
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

// ── #1963: entry-static shared/manual chunk boot smokes ────────────────────
//
// #1948's close-out STOP clause: a `ChunkType::Shared` chunk (auto-detected
// `shared` OR a `[build.manual_chunks]` chunk) that the ENTRY statically
// depends on never loads at runtime — the entry bootstrap used to end in a
// bare synchronous `__jet__.require(entry.id)` with nothing calling
// `__jet__.loadChunk` for that static dependency first, so a real page threw
// `Uncaught Error: Module not found: <id>` at startup. Both smokes below
// reuse the `StaticDistServer`/`spawn_jet_browser`/`browser_eval_json`
// harness from the #1931 lazy-load smoke above, but render visibly into
// `#root` (rather than `console.log`) so a real browser boot is directly
// observable, and assert the dependency chunk's own network request actually
// fired (`performance.getEntriesByType('resource')`).

/// Writes a fixture where the entry statically imports a module that a lazy
/// chunk ALSO imports — `bundler::splitting`'s 2+-chunk-reachability
/// promotion carves that module into a real `"shared"` chunk that the ENTRY
/// itself statically depends on (same shape as `write_shared_promotion_fixture`
/// above, which proves the *build* output; this variant renders its marker
/// into `#root` instead of `console.log`-ing it so a real browser can prove
/// the app actually *boots*). The lazy imports are never triggered — they
/// only need to exist so `bundler::splitting` treats `common.js` as reachable
/// from 2+ chunks; nothing here depends on them ever loading.
fn write_shared_promotion_boot_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import { common } from './common.js';

document.getElementById('root').innerHTML =
  '<div id="output">' + common() + '</div>';

import('./lazy1.js').then((mod) => mod.default());
import('./lazy2.js').then((mod) => mod.default());
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("src/common.js"),
        "export function common() { return 'ENTRY_STATIC_SHARED_MARKER'; }\n",
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

/// #1963 AC1 — an entry-static `shared` chunk (the auto-detected case) must
/// actually load before the entry requires itself: the app boots, renders
/// the shared module's value, and the shared chunk's own script request is
/// observed in the page's resource timing.
///
/// Pre-fix, this reproduces #1948's STOP-clause finding directly: the entry
/// bootstrap was a bare synchronous `__jet__.require(0)` with nothing ever
/// calling `__jet__.loadChunk("shared")` first, so `require(0)` threw
/// `Uncaught Error: Module not found: <common.js's module id>` before
/// `#root` was ever touched — `document.getElementById('output')` would
/// never appear and this test would time out waiting for it.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn splitting_entry_static_shared_chunk_boots_and_renders_in_real_browser() -> Result<()> {
    if !common::chromium_available() {
        eprintln!(
            "skipping splitting_entry_static_shared_chunk_boots_and_renders_in_real_browser: \
             no Chromium available"
        );
        return Ok(());
    }

    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_shared_promotion_boot_fixture(fixture);

    require_success(
        run_jet(fixture, ["build", "--splitting"])?,
        "build --splitting (entry-static shared chunk boot fixture)",
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
        "fixture must force a promoted shared.<hash>.js chunk (WI #1963 repro shape)"
    );

    // Same hard-bound-timeout shape as the #1931 lazy-load smoke above.
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
            // `&&`-guarded lookup: `#output` only exists once the entry's
            // top-level code has actually run (the exact thing pre-fix code
            // never manages to do), so a plain `.textContent` access would
            // throw on `null` while polling before that point.
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
            Some("ENTRY_STATIC_SHARED_MARKER"),
            "app must boot and render the entry's statically-imported shared-chunk dependency \
             (WI #1963: entry bootstrap must loadChunk static deps before require(entry))"
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
                "splitting_entry_static_shared_chunk_boots_and_renders_in_real_browser timed \
                 out after 60s"
            ))
        }
    }
}

/// #1948/#1963 — DOM-rendering variant of `write_manual_chunks_fixture` above:
/// same `node_modules/fakepkg` + `[build.manual_chunks]` vendor-glob shape,
/// but renders visibly into `#root` (instead of `console.log`) so a real
/// browser boot can be proven, closing the live-browser AC that
/// `splitting_manual_chunks_config_routes_glob_matched_module_into_named_chunk_excluded_from_entry`'s
/// doc comment explains was carved out of #1948 pending this fix.
fn write_manual_chunks_boot_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("node_modules/fakepkg")).expect("create node_modules/fakepkg");
    fs::write(
        dir.join("node_modules/fakepkg/package.json"),
        r#"{"name":"fakepkg","version":"1.0.0","main":"index.js"}"#,
    )
    .expect("write fakepkg package.json");
    fs::write(
        dir.join("node_modules/fakepkg/index.js"),
        "export function fakepkg() { return 'FAKEPKG_BOOT_MARKER'; }\n",
    )
    .expect("write fakepkg index.js");

    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import { fakepkg } from 'fakepkg';

document.getElementById('root').innerHTML =
  '<div id="output">' + fakepkg() + '</div>';

import('./lazy1.js').then((mod) => mod.default());
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("src/lazy1.js"),
        "export default function lazy1() { return 'LAZY_ONE_MARKER'; }\n",
    )
    .expect("write lazy1");
}

/// #1963 AC2 — closes #1948's carved-out AC: a manual `[build.manual_chunks]`
/// vendor chunk that the entry statically depends on must load before the
/// entry requires itself, exactly like the auto-detected `shared` case
/// above (manual chunks are `ChunkType::Shared` internally too — see
/// `splitting::split_chunks_with_config`).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn splitting_manual_vendor_chunk_boots_in_real_browser() -> Result<()> {
    if !common::chromium_available() {
        eprintln!(
            "skipping splitting_manual_vendor_chunk_boots_in_real_browser: no Chromium available"
        );
        return Ok(());
    }

    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_manual_chunks_boot_fixture(fixture);
    fs::write(
        fixture.join("jet.toml"),
        "[build.manual_chunks]\nvendor = [\"**/node_modules/fakepkg/**\"]\n",
    )
    .expect("write jet.toml");

    require_success(
        run_jet(fixture, ["build", "--splitting"])?,
        "build --splitting (manual vendor chunk boot fixture)",
    )?;
    let vendor_js_exists = list_files_recursive(&fixture.join("dist/assets"))
        .into_iter()
        .any(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("vendor.") && n.ends_with(".js"))
        });
    assert!(
        vendor_js_exists,
        "fixture must produce a vendor.<hash>.js manual chunk"
    );

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
            Some("FAKEPKG_BOOT_MARKER"),
            "app must boot and render the entry's statically-imported manual vendor chunk \
             dependency (closes #1948's carved-out AC)"
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
            names.iter().any(|n| n.contains("/assets/vendor.")),
            "the vendor chunk's own script request must actually occur: {names:?}"
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
                "splitting_manual_vendor_chunk_boots_in_real_browser timed out after 60s"
            ))
        }
    }
}

// ── WI #1932: splitting default-on for web builds ──────────────────────────
//
// Splitting is no longer an explicit-only opt-in: a web-target `jet build`
// with dynamic imports in its module graph now chunk-splits with NO flags
// at all (`--splitting` still works as an explicit no-op spelling of the
// default; `--no-splitting` is the escape hatch). The flag > config >
// target-default precedence itself is unit-tested directly against
// `build_splitting_enabled` in `src/cli.rs::build_target_validation_table_tests`;
// the tests below are the integration-level proof that the resolved value
// actually reaches a real `jet build` run end to end, plus the emergent
// "nothing to split" fallback and a large-corpus wall-time/size regression
// budget (the #1894 quadratic-regression class).

#[test]
fn default_web_build_splits_automatically_when_graph_has_dynamic_imports() -> Result<()> {
    // Same fixture/shape assertions as
    // `splitting_flag_emits_chunk_files_with_manifest_and_registered_async_chunks`,
    // but with NO `--splitting` flag at all — proving the chunked output is
    // now the default for a web build whose graph has dynamic imports, not
    // something that requires opting in.
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_splitting_fixture(fixture);

    require_success(
        run_jet(fixture, ["build", "--no-minify"])?,
        "build (default flags, no --splitting)",
    )?;

    let dist = fixture.join("dist");
    assert!(
        dist.join("assets").is_dir(),
        "default web build must chunk-split when the graph has dynamic imports"
    );

    let entry_path = list_files_recursive(&dist)
        .into_iter()
        .find(|p| {
            p.parent() == Some(dist.as_path())
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("main.") && n.ends_with(".js"))
        })
        .expect("no top-level main.<hash>.js entry file");
    let entry_code = fs::read_to_string(&entry_path).expect("read entry file");

    assert!(
        !entry_code.contains("LAZY_ONE_MARKER") && !entry_code.contains("LAZY_TWO_MARKER"),
        "default-on entry chunk must exclude lazy-loaded source"
    );
    assert!(
        entry_code.contains("__jet__.dynamicImport("),
        "default-on entry must lower import() to the chunk-aware runtime loader"
    );

    let manifest = extract_json_assignment(&entry_code, "__jet__.chunkManifest")
        .expect("entry must carry a parseable __jet__.chunkManifest assignment");
    let chunks_obj = manifest
        .get("chunks")
        .and_then(Value::as_object)
        .expect("manifest.chunks must be an object");
    for chunk_name in ["chunk-lazy1", "chunk-lazy2"] {
        assert!(
            chunks_obj.contains_key(chunk_name),
            "manifest.chunks missing {chunk_name:?} under default-on splitting: {chunks_obj:?}"
        );
        let chunk_file = chunks_obj[chunk_name]
            .get("file")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("manifest.chunks[{chunk_name:?}].file missing/non-string"));
        assert!(
            dist.join(chunk_file).exists(),
            "manifest names {chunk_file:?} for {chunk_name:?} but no such file was written"
        );
    }

    Ok(())
}

/// Writes a fixture with no dynamic imports at all (only a static import) —
/// the "nothing to split" case `Bundler::generate_split_bundle`'s emergent
/// fallback exists for.
fn write_no_dynamic_import_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import { shared } from './shared.js';

console.log('ENTRY_MARKER', shared());
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("src/shared.js"),
        "export function shared() { return 'SHARED_MARKER'; }\n",
    )
    .expect("write shared");
}

#[test]
fn default_web_build_stays_single_file_and_byte_identical_to_no_splitting_when_graph_has_no_dynamic_imports(
) -> Result<()> {
    // The emergent fallback: a web build defaults `splitting` on, but
    // `Bundler::generate_split_bundle` returns `None` when the graph has no
    // dynamic-import boundaries, so `jet build` (no flags) must fall
    // through to the exact same single-file path as `--no-splitting` —
    // byte-for-byte, not just "also happens to be single file".
    let temp_default = tempfile::tempdir().context("tempdir (default)")?;
    let fixture_default = temp_default.path();
    write_no_dynamic_import_fixture(fixture_default);
    require_success(
        run_jet(fixture_default, ["build"])?,
        "build (default flags)",
    )?;

    let temp_no_splitting = tempfile::tempdir().context("tempdir (--no-splitting)")?;
    let fixture_no_splitting = temp_no_splitting.path();
    write_no_dynamic_import_fixture(fixture_no_splitting);
    require_success(
        run_jet(fixture_no_splitting, ["build", "--no-splitting"])?,
        "build --no-splitting",
    )?;

    let dist_default = fixture_default.join("dist");
    let dist_no_splitting = fixture_no_splitting.join("dist");
    assert!(
        !dist_default.join("assets").exists(),
        "no-dynamic-import graph must not chunk-split even under the new default"
    );

    let default_js: Vec<PathBuf> = list_files_recursive(&dist_default)
        .into_iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("js"))
        .collect();
    let no_splitting_js: Vec<PathBuf> = list_files_recursive(&dist_no_splitting)
        .into_iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("js"))
        .collect();
    assert_eq!(
        default_js.len(),
        1,
        "default build must emit exactly one JS file, got {default_js:?}"
    );
    assert_eq!(
        no_splitting_js.len(),
        1,
        "--no-splitting build must emit exactly one JS file, got {no_splitting_js:?}"
    );

    let default_code = fs::read(&default_js[0]).expect("read default entry file");
    let no_splitting_code = fs::read(&no_splitting_js[0]).expect("read --no-splitting entry file");
    assert_eq!(
        default_code, no_splitting_code,
        "default (splitting attempted, nothing to split) and --no-splitting builds of the same \
         no-dynamic-import graph must produce byte-identical output"
    );
    // The content-hashed filename is derived from these same bytes, so
    // byte-identical output must also carry the identical filename.
    assert_eq!(
        default_js[0].file_name(),
        no_splitting_js[0].file_name(),
        "byte-identical output must also produce the identical content-hashed filename"
    );

    Ok(())
}

#[test]
fn build_splitting_config_false_suppresses_the_web_default() -> Result<()> {
    // `[build].splitting = false` in jet.toml must suppress the new
    // web-target default, even though the graph has dynamic imports that
    // would otherwise trigger automatic splitting.
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_splitting_fixture(fixture);
    fs::write(fixture.join("jet.toml"), "[build]\nsplitting = false\n").expect("write jet.toml");

    require_success(
        run_jet(fixture, ["build"])?,
        "build (jet.toml [build].splitting = false)",
    )?;

    let dist = fixture.join("dist");
    assert!(
        !dist.join("assets").exists(),
        "[build].splitting = false must suppress the web default even with dynamic imports present"
    );
    let js_files: Vec<PathBuf> = list_files_recursive(&dist)
        .into_iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("js"))
        .collect();
    assert_eq!(
        js_files.len(),
        1,
        "config-suppressed build must emit exactly one JS bundle, got {js_files:?}"
    );

    Ok(())
}

/// Generates a synthetic large-corpus fixture: `count` sibling modules, each
/// dynamically imported exactly once from the entry, each carrying its own
/// unique marker string. Script-generated in-test rather than a checked-in
/// fixture tree, so the corpus size is a single tunable constant instead of
/// hundreds of files under `tests/fixtures/`.
fn write_large_dynamic_import_corpus(dir: &Path, count: usize) {
    fs::create_dir_all(dir.join("src")).expect("create src dir");
    let mut entry = String::new();
    for i in 0..count {
        entry.push_str(&format!(
            "import('./mod{i}.js').then((mod) => mod.default());\n"
        ));
        fs::write(
            dir.join(format!("src/mod{i}.js")),
            format!("export default function mod{i}() {{ return 'MOD_{i}_MARKER'; }}\n"),
        )
        .unwrap_or_else(|e| panic!("write src/mod{i}.js: {e}"));
    }
    fs::write(dir.join("src/index.js"), entry).expect("write entry");
}

#[test]
fn large_corpus_default_split_stays_within_chunk_count_size_and_wall_time_budgets() -> Result<()> {
    // Regression coverage for a few hundred dynamic-import boundaries: build
    // wall-time must not blow up (the #1894 quadratic-regression class) and
    // the entry chunk must not leak async payloads. 200 modules measured
    // ~90ms wall-clock locally on a warm run (occasionally ~1.6s on a cold
    // page-cache/first-exec run), both comfortably under the 20s
    // anti-stall threshold that would call for dropping to 100 modules
    // instead. The budget below is a generous ~10s ceiling — >100x the warm
    // measurement, >6x the observed cold outlier — so this only trips on a
    // genuine regression, not machine noise.
    const MODULE_COUNT: usize = 200;
    const WALL_TIME_BUDGET: Duration = Duration::from_secs(10);
    // The manifest/runtime-loader overhead for 200 chunk entries is itself
    // several KB (each `chunks[name] = {file, imports}` entry is ~50-60
    // bytes of JSON); this ceiling is generous enough to never flake on
    // that overhead while still catching a real regression (e.g. async
    // payloads leaking back into the entry).
    const ENTRY_SIZE_CEILING_BYTES: u64 = 200 * 1024;

    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_large_dynamic_import_corpus(fixture, MODULE_COUNT);

    let start = std::time::Instant::now();
    require_success(
        run_jet(fixture, ["build"])?,
        "build (default flags, large corpus)",
    )?;
    let elapsed = start.elapsed();
    assert!(
        elapsed <= WALL_TIME_BUDGET,
        "large-corpus default build took {elapsed:?}, budget is {WALL_TIME_BUDGET:?} \
         (possible quadratic regression, cf. #1894)"
    );

    let dist = fixture.join("dist");
    let entry_path = list_files_recursive(&dist)
        .into_iter()
        .find(|p| {
            p.parent() == Some(dist.as_path())
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("main.") && n.ends_with(".js"))
        })
        .expect("no top-level main.<hash>.js entry file");
    let entry_meta = fs::metadata(&entry_path).expect("stat entry file");
    assert!(
        entry_meta.len() <= ENTRY_SIZE_CEILING_BYTES,
        "entry file is {} bytes, ceiling is {ENTRY_SIZE_CEILING_BYTES} bytes (entry must exclude \
         async chunk payloads)",
        entry_meta.len()
    );

    let entry_code = fs::read_to_string(&entry_path).expect("read entry file");
    for i in 0..MODULE_COUNT {
        assert!(
            !entry_code.contains(&format!("MOD_{i}_MARKER")),
            "entry chunk must exclude the async payload for mod{i}.js (found MOD_{i}_MARKER)"
        );
    }

    // Chunk count == boundary count: every module here is dynamically
    // imported exactly once directly from the entry (no shared-chunk
    // promotion applies — that only kicks in for modules reachable from
    // more than one split point), so partitioning must produce exactly
    // `MODULE_COUNT` chunk files under `dist/assets/` (counted directly by
    // extension, sidestepping any JSON-manifest minification concerns).
    let asset_js_files: Vec<PathBuf> = list_files_recursive(&dist.join("assets"))
        .into_iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("js"))
        .collect();
    assert_eq!(
        asset_js_files.len(),
        MODULE_COUNT,
        "expected exactly {MODULE_COUNT} chunk files under dist/assets/, got {}",
        asset_js_files.len()
    );

    // Manifest integrity, checked via a second `--no-minify` build of the
    // identical corpus in its own tempdir (fresh dir so stray files from
    // the timed build above can't inflate this build's own dist/assets/
    // listing) — every module id must map to a chunk, and every chunk must
    // name a file that actually exists on disk.
    let temp_raw = tempfile::tempdir().context("tempdir (no-minify)")?;
    let fixture_raw = temp_raw.path();
    write_large_dynamic_import_corpus(fixture_raw, MODULE_COUNT);
    require_success(
        run_jet(fixture_raw, ["build", "--no-minify"])?,
        "build --no-minify (large corpus, manifest integrity)",
    )?;
    let raw_dist = fixture_raw.join("dist");
    let raw_entry_path = list_files_recursive(&raw_dist)
        .into_iter()
        .find(|p| {
            p.parent() == Some(raw_dist.as_path())
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("main.") && n.ends_with(".js"))
        })
        .expect("no top-level main.<hash>.js entry file (no-minify build)");
    let raw_entry_code = fs::read_to_string(&raw_entry_path).expect("read entry file (no-minify)");
    let manifest = extract_json_assignment(&raw_entry_code, "__jet__.chunkManifest")
        .expect("entry must carry a parseable __jet__.chunkManifest assignment");
    let chunks_obj = manifest
        .get("chunks")
        .and_then(Value::as_object)
        .expect("manifest.chunks must be an object");
    let module_chunks_obj = manifest
        .get("moduleChunks")
        .and_then(Value::as_object)
        .expect("manifest.moduleChunks must be an object");
    assert_eq!(
        chunks_obj.len(),
        MODULE_COUNT,
        "expected exactly {MODULE_COUNT} manifest.chunks entries, got {}",
        chunks_obj.len()
    );
    assert_eq!(
        module_chunks_obj.len(),
        MODULE_COUNT,
        "expected exactly {MODULE_COUNT} manifest.moduleChunks entries, got {}",
        module_chunks_obj.len()
    );
    for (chunk_name, chunk_meta) in chunks_obj {
        let file = chunk_meta
            .get("file")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("manifest.chunks[{chunk_name:?}].file missing/non-string"));
        assert!(
            raw_dist.join(file).exists(),
            "manifest names {file:?} for {chunk_name:?} but no such file exists on disk"
        );
    }
    for (module_id, chunk_name) in module_chunks_obj {
        let chunk_name_str = chunk_name
            .as_str()
            .unwrap_or_else(|| panic!("moduleChunks[{module_id:?}] is not a string"));
        assert!(
            chunks_obj.contains_key(chunk_name_str),
            "moduleChunks[{module_id:?}] = {chunk_name_str:?} has no matching entry in manifest.chunks"
        );
    }

    Ok(())
}

/// Writes a fixture combining the two real-world path-spelling sources named
/// in #1941: a pnpm-style `node_modules` symlink (the real package directory
/// lives under a content-addressed `.pnpm/<name>@<version>/node_modules/<name>/`
/// path, and `node_modules/<name>` is a *symlink* into it) and a jet.toml
/// `[alias]` rewrite. The entry statically imports the symlinked package
/// (single reference — stays inlined into the entry) and dynamically
/// `import()`s `lazy.js`, which statically imports its own private dep
/// through the alias (also single reference — stays inlined into the async
/// chunk it's split into, not promoted to a separate shared chunk).
#[cfg(unix)]
fn write_symlink_and_alias_fixture(dir: &Path) {
    // pnpm-style dependency layout: the real package directory is nested
    // under `.pnpm/<name>@<version>/node_modules/<name>/`, and
    // `node_modules/<name>` is a relative symlink into it. The resolved
    // import path for `shared-dep` therefore never spells the same as any
    // path a naive equality-based BFS would already have visited.
    let real_pkg = dir.join("node_modules/.pnpm/shared-dep@1.0.0/node_modules/shared-dep");
    fs::create_dir_all(&real_pkg).expect("create real pnpm store dir");
    fs::write(
        real_pkg.join("package.json"),
        r#"{"name":"shared-dep","version":"1.0.0","main":"index.js"}"#,
    )
    .expect("write shared-dep package.json");
    fs::write(
        real_pkg.join("index.js"),
        "export function shared() { return 'STORE_SHARED_MARKER'; }\n",
    )
    .expect("write shared-dep index.js");
    std::os::unix::fs::symlink(
        ".pnpm/shared-dep@1.0.0/node_modules/shared-dep",
        dir.join("node_modules/shared-dep"),
    )
    .expect("create pnpm-style node_modules symlink");

    // jet.toml `[alias]`: a second real-world path-spelling source —
    // `@lib/private.js` resolves through config-driven rewriting, not a
    // plain relative import or node_modules walk-up.
    fs::write(
        dir.join("jet.toml"),
        "[alias]\n\"@lib/\" = \"./aliased/\"\n",
    )
    .expect("write jet.toml");
    fs::create_dir_all(dir.join("aliased")).expect("create aliased dir");
    fs::write(
        dir.join("aliased/private.js"),
        "export function privateHelper() { return 'PRIVATE_DEP_MARKER'; }\n",
    )
    .expect("write aliased/private.js");

    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import { shared } from 'shared-dep';

console.log('ENTRY_MARKER', shared());

import('./lazy.js').then((mod) => mod.default());
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("src/lazy.js"),
        r#"import { privateHelper } from '@lib/private.js';

export default function lazy() {
    return 'LAZY_MARKER:' + privateHelper();
}
"#,
    )
    .expect("write lazy");
}

/// Regression coverage for #1941: chunk partitioning used to key module
/// identity by `PathBuf` equality, which collapses on real-world graphs
/// where the same logical module is reachable through more than one path
/// spelling — a pnpm-style `node_modules` symlink, a jet.toml `[alias]`
/// rewrite. Production evidence showed ~5 duplicate `__jet__.define(`
/// emissions plus hundreds of orphaned modules silently flooding into the
/// entry chunk with zero shared-chunk extraction. Partitioning is now keyed
/// by `CompiledModule::id: usize` end-to-end, so this fixture exercises both
/// real-world path-spelling sources at once and asserts the failure
/// signature is gone: no module's `__jet__.define(` is emitted more than
/// once across entry+chunks, and every module lands in exactly the chunk
/// its reachability says it should.
#[cfg(unix)]
#[test]
fn splitting_survives_pnpm_symlink_and_alias_without_duplicating_or_orphaning_modules() -> Result<()>
{
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_symlink_and_alias_fixture(fixture);

    require_success(
        run_jet(fixture, ["build", "--splitting", "--no-minify"])?,
        "build --splitting --no-minify (pnpm-symlink + alias fixture)",
    )?;

    let dist = fixture.join("dist");
    let entry_path = list_files_recursive(&dist)
        .into_iter()
        .find(|p| {
            p.parent() == Some(dist.as_path())
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("main.") && n.ends_with(".js"))
        })
        .expect("no top-level main.<hash>.js entry file");
    let entry_code = fs::read_to_string(&entry_path).expect("read entry file");

    // Entry must contain its own source plus the symlinked static dep
    // (single-reference — stays inlined, not promoted to a shared chunk).
    assert!(
        entry_code.contains("ENTRY_MARKER") && entry_code.contains("STORE_SHARED_MARKER"),
        "entry chunk must contain its own source plus the symlinked static dep"
    );
    // Entry must exclude the entire lazy subtree: the split-point module
    // and its alias-resolved private dep.
    assert!(
        !entry_code.contains("LAZY_MARKER") && !entry_code.contains("PRIVATE_DEP_MARKER"),
        "entry chunk must exclude the lazy module and its alias-resolved private dep; entry={}",
        entry_path.display()
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

    assert_eq!(
        chunks_obj.len(),
        1,
        "expected exactly one async chunk (\"chunk-lazy\"), got {chunks_obj:?}"
    );
    let lazy_chunk_meta = chunks_obj
        .get("chunk-lazy")
        .unwrap_or_else(|| panic!("manifest.chunks missing \"chunk-lazy\": {chunks_obj:?}"));
    let lazy_chunk_file = lazy_chunk_meta
        .get("file")
        .and_then(Value::as_str)
        .expect("manifest.chunks[\"chunk-lazy\"].file missing/non-string");
    let lazy_chunk_path = dist.join(lazy_chunk_file);
    assert!(
        lazy_chunk_path.exists(),
        "manifest names {lazy_chunk_file:?} for chunk-lazy but no such file was written"
    );
    let lazy_chunk_code = fs::read_to_string(&lazy_chunk_path).expect("read chunk-lazy file");

    // The async chunk must carry its full subtree together: the split-point
    // module plus its private single-reference static dep (resolved through
    // the jet.toml alias) — not split apart, not orphaned back into the
    // entry.
    assert!(
        lazy_chunk_code.contains("LAZY_MARKER") && lazy_chunk_code.contains("PRIVATE_DEP_MARKER"),
        "chunk-lazy must carry the lazy module and its alias-resolved private dep together; code={lazy_chunk_code}"
    );

    // moduleChunks only records modules assigned to a *named* (non-entry)
    // chunk (see `build_chunk_manifest_js`), so exactly the 2 chunk-lazy
    // members (lazy.js + its private dep) are expected here — the 2
    // entry-inlined modules (index.js + the symlinked shared-dep) are not.
    assert_eq!(
        module_chunks_obj.len(),
        2,
        "expected exactly 2 moduleChunks entries (lazy.js + its alias-resolved private dep), \
         got {module_chunks_obj:?}"
    );
    for chunk_name in module_chunks_obj.values() {
        assert_eq!(
            chunk_name.as_str(),
            Some("chunk-lazy"),
            "every moduleChunks entry must point at chunk-lazy, got {module_chunks_obj:?}"
        );
    }

    // Zero-duplication assertion (#1941's core regression signature): exactly
    // one `__jet__.define(` per unique compiled module — entry index.js, the
    // symlinked shared-dep, lazy.js, and the alias-resolved private.js (4
    // total) — counted across every emitted file. Pre-#1941 `PathBuf`-keyed
    // BFS could double-count a module whose id was reachable under more than
    // one path spelling; it is now structurally impossible under id-keying.
    const EXPECTED_UNIQUE_MODULES: usize = 4;
    let total_defines = entry_code.matches("__jet__.define(").count()
        + lazy_chunk_code.matches("__jet__.define(").count();
    assert_eq!(
        total_defines, EXPECTED_UNIQUE_MODULES,
        "expected exactly {EXPECTED_UNIQUE_MODULES} __jet__.define( calls across entry+chunks \
         (one per unique module: entry, symlinked shared-dep, lazy, alias-resolved private), \
         got {total_defines}"
    );

    Ok(())
}

/// WI #1995 round 4/5/6 — the pre-transform survivors-only transform filter
/// (default-on since round 6; opt out via `JET_NO_SURVIVOR_FILTER=1`) must
/// be byte-identical to the escape-hatch (filter off) build: same `dist/`
/// file set, same bytes, for a real `--splitting` build (the filter only
/// skips transform *work* for modules a raw-source liveness pre-pass proves
/// dead; it must never change what actually ships).
///
/// Uses `--no-minify` (same reason as the manifest-parsing tests above:
/// `build_chunk_manifest_js` only emits strict, fully-quoted JSON before
/// the minifier rewrites unquoted-identifier keys) and, for the top-level
/// entry file specifically, compares `normalize_entry_code`'s output
/// rather than raw bytes — see this test's and that helper's doc comments
/// for why raw entry bytes are not currently a stable target. Both builds
/// run from *one* fixture directory (via `-o`, not 2 separate tempdirs):
/// `--no-minify` output keeps a `// Module N: <absolute path>` comment
/// inline in chunk JS (not just `.map`), so 2 different tempdirs would
/// make even chunk files legitimately differ for a reason unrelated to
/// this WI. Every non-entry `dist/` file is compared byte-for-byte.
#[test]
fn splitting_survivor_filter_is_byte_identical_to_escape_hatch() -> Result<()> {
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_splitting_fixture(fixture);

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
         escape-hatch (JET_NO_SURVIVOR_FILTER=1, filter off) build"
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

/// WI #1994 — the "Write output" loop's per-chunk `.js` + `.map` file
/// writes are now parallelized (rayon over `HashedChunk`s) instead of
/// serial; `JET_SERIAL_CHUNK_WRITES=1` is the escape hatch this test uses
/// as the serial ground truth to diff against. Reuses
/// `write_large_dynamic_import_corpus` (originally #1894's
/// quadratic-regression fixture) to get a build with many independent
/// chunks, so the parallel write path actually fans out across more than
/// one file.
///
/// Both builds run from *one* fixture directory (via `-o`, not 2 separate
/// tempdirs) — same reason as
/// `splitting_survivor_filter_is_byte_identical_to_escape_hatch` above:
/// `--no-minify` output keeps a `// Module N: <absolute path>` comment
/// inline in chunk JS, so 2 different tempdirs would make even chunk files
/// legitimately differ for a reason unrelated to this WI. Also mirrors that
/// test's entry-file handling (`normalize_entry_code` + parsed-manifest
/// comparison instead of raw byte equality): the entry's content hash can
/// legitimately differ run-to-run for the same pre-existing,
/// separately-tracked reason documented on `normalize_entry_code` (manifest
/// key order is not stable across separate process invocations) — that is
/// unrelated to chunk *write* parallelization, which only changes I/O
/// order, not any computed content (chunk minify/hashing/manifest
/// construction all already ran, in `hashed_chunks`' original list order,
/// before the write loop).
#[test]
fn splitting_parallel_chunk_writes_match_serial_escape_hatch() -> Result<()> {
    const MODULE_COUNT: usize = 80;

    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_large_dynamic_import_corpus(fixture, MODULE_COUNT);

    fn build(fixture: &Path, out_dir: &str, serial: bool) -> Result<PathBuf> {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_jet"));
        cmd.args(["build", "--splitting", "--no-minify", "-o", out_dir])
            .current_dir(fixture);
        if serial {
            cmd.env("JET_SERIAL_CHUNK_WRITES", "1");
        }
        let output = cmd
            .output()
            .context("run jet build --splitting --no-minify")?;
        require_success(
            output,
            if serial {
                "build --splitting --no-minify (JET_SERIAL_CHUNK_WRITES=1)"
            } else {
                "build --splitting --no-minify (default, parallel chunk writes)"
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

    let dist_parallel = build(fixture, "dist-parallel", false)?;
    let dist_serial = build(fixture, "dist-serial", true)?;

    // Sanity: this fixture must actually produce many chunks, or the
    // parallel write path never fans out across more than one file.
    let count_ext = |dist: &Path, ext: &str| -> usize {
        list_files_recursive(&dist.join("assets"))
            .into_iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some(ext))
            .count()
    };
    assert_eq!(
        count_ext(&dist_parallel, "js"),
        MODULE_COUNT,
        "expected {MODULE_COUNT} chunk .js files under dist-parallel/assets/"
    );
    assert_eq!(
        count_ext(&dist_parallel, "map"),
        MODULE_COUNT,
        "expected {MODULE_COUNT} chunk .map files under dist-parallel/assets/ \
         (default sourcemap mode is external)"
    );

    let is_entry = |rel: &str| -> bool {
        rel.rsplit('/')
            .next()
            .is_some_and(|name| name.starts_with("main."))
    };
    let (parallel_entry, parallel_rest): (Vec<_>, Vec<_>) = sorted_entries(&dist_parallel)
        .into_iter()
        .partition(|(rel, _)| is_entry(rel));
    let (serial_entry, serial_rest): (Vec<_>, Vec<_>) = sorted_entries(&dist_serial)
        .into_iter()
        .partition(|(rel, _)| is_entry(rel));

    // Entry file (`main.<hash>.js` + `.js.map`): the content hash itself
    // can differ run-to-run for a pre-existing, separately-tracked reason
    // unrelated to this WI (see doc comment) — require both builds to
    // still emit exactly one entry `.js` + one entry `.js.map`, and use
    // the 2 (possibly differing) filenames to normalize `index.html`'s
    // `<script src>` reference below.
    assert_eq!(
        parallel_entry.len(),
        2,
        "expected exactly main.<hash>.js + .js.map, got {parallel_entry:?}"
    );
    assert_eq!(
        serial_entry.len(),
        2,
        "expected exactly main.<hash>.js + .js.map, got {serial_entry:?}"
    );
    fn entry_js_path(entries: &[(String, PathBuf)]) -> &Path {
        entries
            .iter()
            .find(|(rel, _)| rel.ends_with(".js"))
            .map(|(_, p)| p.as_path())
            .expect("main.<hash>.js missing")
    }
    let entry_parallel_path = entry_js_path(&parallel_entry);
    let entry_serial_path = entry_js_path(&serial_entry);
    let entry_parallel_name = entry_parallel_path
        .file_name()
        .and_then(OsStr::to_str)
        .expect("entry filename utf-8")
        .to_string();
    let entry_serial_name = entry_serial_path
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
    // it necessarily follows the entry's own (possibly differing) hash.
    let parallel_rest_rel: Vec<&String> = parallel_rest.iter().map(|(r, _)| r).collect();
    let serial_rest_rel: Vec<&String> = serial_rest.iter().map(|(r, _)| r).collect();
    assert_eq!(
        parallel_rest_rel, serial_rest_rel,
        "parallel (default) and serial (JET_SERIAL_CHUNK_WRITES=1) chunk-write \
         builds must emit the same non-entry dist/ file set"
    );
    for ((rel, pp), (_, sp)) in parallel_rest.iter().zip(serial_rest.iter()) {
        let pb = fs::read(pp).with_context(|| format!("read {}", pp.display()))?;
        let sb = fs::read(sp).with_context(|| format!("read {}", sp.display()))?;
        if rel == "index.html" {
            let ps_text = String::from_utf8(pb).context("index.html must be utf-8")?;
            let ss_text = String::from_utf8(sb).context("index.html must be utf-8")?;
            assert_eq!(
                ps_text.replace(&entry_parallel_name, "<<ENTRY>>"),
                ss_text.replace(&entry_serial_name, "<<ENTRY>>"),
                "dist/index.html must be byte-identical (modulo the entry's \
                 own content-hashed filename) between the parallel (default) \
                 and serial (JET_SERIAL_CHUNK_WRITES=1) chunk-write builds"
            );
            continue;
        }
        assert_eq!(
            pb, sb,
            "dist/{rel} must be byte-identical between the parallel (default) \
             and serial (JET_SERIAL_CHUNK_WRITES=1) chunk-write builds"
        );
    }

    let entry_parallel_code =
        fs::read_to_string(entry_parallel_path).context("read parallel entry")?;
    let entry_serial_code = fs::read_to_string(entry_serial_path).context("read serial entry")?;
    let (entry_parallel_norm, manifest_parallel) = normalize_entry_code(&entry_parallel_code);
    let (entry_serial_norm, manifest_serial) = normalize_entry_code(&entry_serial_code);
    assert_eq!(
        entry_parallel_norm, entry_serial_norm,
        "entry file must be byte-identical outside the chunkManifest \
         assignment (and its own sourceMappingURL comment) between the \
         parallel (default) and serial (JET_SERIAL_CHUNK_WRITES=1) \
         chunk-write builds"
    );
    assert_eq!(
        manifest_parallel, manifest_serial,
        "chunkManifest must be the same JSON value (key order ignored — see \
         normalize_entry_code's doc comment) between the parallel (default) \
         and serial (JET_SERIAL_CHUNK_WRITES=1) chunk-write builds"
    );

    Ok(())
}
// HANDWRITE-END
