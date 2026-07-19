// HANDWRITE-BEGIN gap="missing-generator:unit-test:3d700a0c" tracker="pending-tracker" reason="Integration test: the crawl-time import/export extraction fast path (issue #1997) must produce byte-identical `jet build --splitting` output to the pre-existing tree-sitter-only extraction path; drives 2 real `jet build --splitting --no-minify` subprocesses (default fast path vs. the JET_NO_FAST_IMPORT_SCAN=1 escape hatch) over one synthetic fixture exercising default/named/aliased/namespace imports, a multi-line wrapped import, and a dynamic import() lazy chunk, then diffs the two dist/ trees byte-for-byte (entry file compared via code_splitting.rs's established normalize_entry_code technique, since build_chunk_manifest_js's manifest key order is not currently a stable byte-identity target across any 2 splitting builds, not just this WI's escape hatch)."
//! `bundler::imports::extract_imports_fast` byte-identity coverage (#1997,
//! child of the beat-vite epic #1990): a `jet build --splitting` of a small
//! synthetic fixture must produce the exact same `dist/` output whether the
//! string-scan fast path is in effect (the default) or bypassed via the
//! `JET_NO_FAST_IMPORT_SCAN=1` escape hatch (forcing every crawled module
//! through the pre-existing tree-sitter walk).
//!
//! `normalize_entry_code` (chunk-manifest-JSON-key-order normalization +
//! `sourceMappingURL` blanking) is duplicated from
//! `tests/build/code_splitting.rs` rather than extracted into a shared
//! helper, matching `tests/build/entry_flatten.rs`'s own stated precedent
//! that cross-file extraction here would be its own out-of-scope refactor.

use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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

/// A small fixture exercising the fast path's representative shapes: a
/// default import, a named import with an alias, a namespace import, a
/// multi-line (prettier-wrapped) named import, and a dynamic `import()`
/// (forcing a `--splitting` lazy chunk so the call-expression scanner is
/// exercised end-to-end, not just the statement scanner).
fn write_fast_import_scan_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::write(
        dir.join("src/index.js"),
        r#"import { helper, helper2 as aliasedHelper } from './helper.js';
import Default from './default-export.js';
import * as ns from './namespace.js';

console.log('ENTRY_MARKER', helper(), aliasedHelper(), Default(), ns.value);

import('./lazy.js').then((mod) => mod.default());
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("src/helper.js"),
        "export function helper() { return 'HELPER_MARKER'; }\nexport function helper2() { return 'HELPER2_MARKER'; }\n",
    )
    .expect("write helper");
    fs::write(
        dir.join("src/default-export.js"),
        "export default function defaultExport() { return 'DEFAULT_MARKER'; }\n",
    )
    .expect("write default-export");
    fs::write(
        dir.join("src/namespace.js"),
        "export const value = 'NAMESPACE_MARKER';\n",
    )
    .expect("write namespace");
    fs::write(
        dir.join("src/lazy.js"),
        r#"import {
  helper,
  helper2,
} from './helper.js';

export default function lazy() {
  return 'LAZY_MARKER:' + helper() + helper2();
}
"#,
    )
    .expect("write lazy");
}

/// Normalizes an entry file's code for cross-build byte-identity
/// comparison: blanks the trailing `//# sourceMappingURL=...` comment
/// (embeds the entry's own content hash) and splits the
/// `__jet__.chunkManifest = {...}` assignment out into a separately
/// returned, parsed `Value` (replaced in the code by a fixed placeholder).
///
/// Needed because `build_chunk_manifest_js`'s `chunks`/`moduleChunks` key
/// order is not currently stable run-to-run for otherwise byte-identical
/// input — see `tests/build/code_splitting.rs::normalize_entry_code`'s doc
/// comment (this copy is behaviorally identical).
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

/// AC3: `jet build --splitting` of the fixture above must emit an
/// identical `dist/` tree whether the string-scan fast path
/// (`imports::extract_imports_fast`) is in effect (default) or bypassed via
/// `JET_NO_FAST_IMPORT_SCAN=1`.
#[test]
fn fast_import_scan_default_is_byte_identical_to_escape_hatch() -> Result<()> {
    let temp = tempfile::tempdir().context("tempdir")?;
    let fixture = temp.path();
    write_fast_import_scan_fixture(fixture);

    fn build(fixture: &Path, out_dir: &str, no_fast_scan: bool) -> Result<PathBuf> {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_jet"));
        cmd.args(["build", "--splitting", "--no-minify", "-o", out_dir])
            .current_dir(fixture);
        if no_fast_scan {
            cmd.env("JET_NO_FAST_IMPORT_SCAN", "1");
        }
        let output = cmd
            .output()
            .context("run jet build --splitting --no-minify")?;
        require_success(
            output,
            if no_fast_scan {
                "build --splitting --no-minify (JET_NO_FAST_IMPORT_SCAN=1)"
            } else {
                "build --splitting --no-minify (default, fast import scan on)"
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

    let dist_fast = build(fixture, "dist-fast", false)?;
    let dist_fallback = build(fixture, "dist-fallback", true)?;

    let is_entry = |rel: &str| -> bool {
        rel.rsplit('/')
            .next()
            .is_some_and(|name| name.starts_with("main."))
    };
    let (fast_entry, fast_rest): (Vec<_>, Vec<_>) = sorted_entries(&dist_fast)
        .into_iter()
        .partition(|(rel, _)| is_entry(rel));
    let (fallback_entry, fallback_rest): (Vec<_>, Vec<_>) = sorted_entries(&dist_fallback)
        .into_iter()
        .partition(|(rel, _)| is_entry(rel));

    assert_eq!(
        fast_entry.len(),
        2,
        "expected exactly main.<hash>.js + .js.map, got {fast_entry:?}"
    );
    assert_eq!(
        fallback_entry.len(),
        2,
        "expected exactly main.<hash>.js + .js.map, got {fallback_entry:?}"
    );
    fn entry_js_path(entries: &[(String, PathBuf)]) -> &Path {
        entries
            .iter()
            .find(|(rel, _)| rel.ends_with(".js"))
            .map(|(_, p)| p.as_path())
            .expect("main.<hash>.js missing")
    }
    let entry_fast_path = entry_js_path(&fast_entry);
    let entry_fallback_path = entry_js_path(&fallback_entry);
    let entry_fast_name = entry_fast_path
        .file_name()
        .and_then(OsStr::to_str)
        .expect("entry filename utf-8")
        .to_string();
    let entry_fallback_name = entry_fallback_path
        .file_name()
        .and_then(OsStr::to_str)
        .expect("entry filename utf-8")
        .to_string();

    // Non-entry files (chunk JS + every .map + index.html): exact
    // relative-path set and byte-identical content. Both builds share one
    // fixture directory, so the `--no-minify` `// Module N: <absolute
    // path>` comment chunk JS inlines is identical either way.
    // `index.html` alone gets its `<script src="./main.<hash>.js">`
    // reference normalized first, since it necessarily follows the entry's
    // own (potentially differing, per `normalize_entry_code`'s doc comment)
    // hash.
    let fast_rest_rel: Vec<&String> = fast_rest.iter().map(|(r, _)| r).collect();
    let fallback_rest_rel: Vec<&String> = fallback_rest.iter().map(|(r, _)| r).collect();
    assert_eq!(
        fast_rest_rel, fallback_rest_rel,
        "the fast import scan must emit the same non-entry dist/ file set as \
         the JET_NO_FAST_IMPORT_SCAN=1 escape-hatch build"
    );
    for ((rel, fp), (_, up)) in fast_rest.iter().zip(fallback_rest.iter()) {
        let fb = fs::read(fp).with_context(|| format!("read {}", fp.display()))?;
        let ub = fs::read(up).with_context(|| format!("read {}", up.display()))?;
        if rel == "index.html" {
            let fs_text = String::from_utf8(fb).context("index.html must be utf-8")?;
            let us_text = String::from_utf8(ub).context("index.html must be utf-8")?;
            assert_eq!(
                fs_text.replace(&entry_fast_name, "<<ENTRY>>"),
                us_text.replace(&entry_fallback_name, "<<ENTRY>>"),
                "dist/index.html must be byte-identical (modulo the entry's \
                 own content-hashed filename) between the fast import scan \
                 and the JET_NO_FAST_IMPORT_SCAN=1 escape-hatch build"
            );
            continue;
        }
        assert_eq!(
            fb, ub,
            "dist/{rel} must be byte-identical between the fast import scan \
             and the JET_NO_FAST_IMPORT_SCAN=1 escape-hatch build"
        );
    }

    let entry_fast_code = fs::read_to_string(entry_fast_path).context("read fast entry")?;
    let entry_fallback_code =
        fs::read_to_string(entry_fallback_path).context("read fallback entry")?;
    let (entry_fast_norm, manifest_fast) = normalize_entry_code(&entry_fast_code);
    let (entry_fallback_norm, manifest_fallback) = normalize_entry_code(&entry_fallback_code);
    assert_eq!(
        entry_fast_norm, entry_fallback_norm,
        "entry file must be byte-identical outside the chunkManifest \
         assignment (and its own sourceMappingURL comment) between the fast \
         import scan and the JET_NO_FAST_IMPORT_SCAN=1 escape-hatch build"
    );
    assert_eq!(
        manifest_fast, manifest_fallback,
        "chunkManifest must be the same JSON value (key order ignored) \
         between the fast import scan and the JET_NO_FAST_IMPORT_SCAN=1 \
         escape-hatch build"
    );

    Ok(())
}
// HANDWRITE-END
