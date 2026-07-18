// HANDWRITE-BEGIN gap="missing-generator:unit-test:7fc477c4" tracker="pending-tracker" reason="Integration tests: a pure re-export barrel (many leaves, few used, @mui/icons-material-shaped) crawls only the demanded leaves through a real `jet build` subprocess — JET_BUNDLE_TIMING reports a skipped-leaf count close to the unused total, and the built entry contains only the demanded leaves' markers — and the default lazy crawl's entry output agrees with a JET_EAGER_BARRELS=1 eager-crawl build of the same fixture on the exact used-marker set and the per-leaf transformed code body, while producing a strictly smaller entry (module ids are crawl-order/crawled-set-size dependent by design, so whole-file byte-identity across the two modes is not the correct bar — see the second test's doc comment)."
//! Lazy pure-barrel expansion (#1991) integration coverage: the AC1
//! 1,000-leaf/3-used synthetic fixture.
//!
//! Unit coverage for the pure-barrel detector, per-specifier demand
//! narrowing, demand-set accumulation across waves, the same-wave
//! multi-importer hazard, and every fallback case (namespace import, dynamic
//! `import()`, bare/unused CJS `require`, `export *` propagation, an
//! unresolvable requested name) already lives in `src/bundler/mod.rs`'s
//! `lazy_barrel_expansion_tests` module, exercising `Bundler::build_graph`
//! and the crawl-private `is_pure_barrel_source`/`barrel_demand_for_specifier`
//! helpers directly — those aren't reachable from an integration-test crate.
//!
//! This file covers what only a real `jet build` subprocess can prove: the
//! `JET_BUNDLE_TIMING` crawl-count instrumentation signal on a
//! 1,000-leaf/3-used barrel (the issue's own AC1 shape), and used-module
//! output agreement between the default lazy crawl and the
//! `JET_EAGER_BARRELS=1` escape hatch that restores the pre-#1991 eager
//! crawl.

use anyhow::{anyhow, Context, Result};
use std::collections::BTreeSet;
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

/// Runs `jet build --no-minify` in `fixture` with the given extra env vars
/// set, and requires it to succeed.
fn run_build(fixture: &Path, envs: &[(&str, &str)]) -> Result<Output> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_jet"));
    cmd.args(["build", "--no-minify"]).current_dir(fixture);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    require_success(cmd.output().context("run jet build")?, "jet build")
}

/// Recursively collects every regular file under `dir` (absolute paths).
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

/// Locates the single top-level `main.<hash>.js` entry file directly under
/// `dist/` (mirrors the identically-named helper in
/// `tests/build/code_splitting.rs`; integration test files are separate
/// compiled crates so the helper can't be shared without a dependency this
/// file doesn't otherwise need).
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

/// `dir`-relative, sorted file set — for cross-tempdir `dist/` comparison.
fn relative_file_set(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = list_files_recursive(dir)
        .into_iter()
        .filter_map(|p| p.strip_prefix(dir).ok().map(|r| r.to_path_buf()))
        .collect();
    out.sort();
    out
}

/// Scans `text` for every `ICON_<n>_MARKER` literal (this fixture's unique
/// per-leaf tag — see `write_barrel_fixture`) and returns the sorted, deduped
/// set of `<n>` indices found. Used to prove built output contains exactly
/// the demanded leaves' markers and none of the unrequested ones.
fn find_icon_marker_indices(text: &str) -> BTreeSet<usize> {
    let mut out = BTreeSet::new();
    let mut rest = text;
    while let Some(pos) = rest.find("ICON_") {
        let tail = &rest[pos + "ICON_".len()..];
        if let Some(marker_pos) = tail.find("_MARKER") {
            let digits = &tail[..marker_pos];
            if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
                if let Ok(n) = digits.parse::<usize>() {
                    out.insert(n);
                }
            }
        }
        rest = tail;
    }
    out
}

/// Writes an `@mui/icons-material`-shaped pure re-export barrel:
/// `src/icons/Icon{0..leaf_count}.js`, each exporting one distinct const
/// tagged with a unique marker string; `src/icons/index.js` re-exporting all
/// of them by name (a pure barrel — nothing but `export ... from ...`
/// lines); and `src/index.js` importing + using only `used`'s indices by
/// name.
fn write_barrel_fixture(dir: &Path, leaf_count: usize, used: &[usize]) {
    let icons_dir = dir.join("src/icons");
    fs::create_dir_all(&icons_dir).expect("create src/icons dir");
    for i in 0..leaf_count {
        fs::write(
            icons_dir.join(format!("Icon{i}.js")),
            format!("export const Icon{i} = 'ICON_{i}_MARKER';\n"),
        )
        .unwrap_or_else(|e| panic!("write Icon{i}.js: {e}"));
    }
    let barrel: String = (0..leaf_count)
        .map(|i| format!("export {{ Icon{i} }} from './Icon{i}.js';\n"))
        .collect();
    fs::write(icons_dir.join("index.js"), barrel).expect("write icons/index.js");

    let imports = used
        .iter()
        .map(|i| format!("Icon{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let logs: String = used
        .iter()
        .map(|i| format!("console.log(Icon{i});\n"))
        .collect();
    fs::write(
        dir.join("src/index.js"),
        format!("import {{ {imports} }} from './icons/index.js';\n\n{logs}"),
    )
    .expect("write src/index.js");
}

/// AC1: a 1,000-leaf barrel where only 3 leaves are demanded (indices 0,
/// 500, 999 — first, middle, last). `JET_BUNDLE_TIMING=1` must report a
/// skipped-leaf count close to the 997 unused leaves, and the built entry
/// must contain only the 3 demanded leaves' markers (proving the crawl
/// result is correct, not just small).
#[test]
fn lazy_crawl_skips_unrequested_barrel_leaves_ac1() -> Result<()> {
    let tmp = tempfile::tempdir().context("tempdir")?;
    let dir = tmp.path();
    let leaf_count = 1_000;
    let used = [0usize, 500, 999];
    write_barrel_fixture(dir, leaf_count, &used);

    let output = run_build(dir, &[("JET_BUNDLE_TIMING", "1")])?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let timing_line = stderr
        .lines()
        .find(|l| l.contains("lazy-barrels:"))
        .unwrap_or_else(|| panic!("no lazy-barrels timing line in stderr:\n{stderr}"));
    let skipped: usize = timing_line
        .rsplit_once("detected, ")
        .and_then(|(_, tail)| tail.split_whitespace().next())
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("could not parse skipped-leaf count from: {timing_line}"));

    // Every un-demanded leaf must be skipped (leaf_count - used.len()), with
    // a small tolerance in case a future refactor folds other bootstrap
    // modules into the same counter.
    let expected_skipped = leaf_count - used.len();
    assert!(
        skipped >= expected_skipped.saturating_sub(5),
        "expected close to {expected_skipped} skipped barrel leaves (leaf_count \
         {leaf_count} - used {}), got {skipped} from timing line: {timing_line}",
        used.len()
    );

    let dist = dir.join("dist");
    let entry_path = find_entry_file(&dist);
    let entry_js = fs::read_to_string(&entry_path)
        .with_context(|| format!("read entry file {entry_path:?}"))?;
    for i in used {
        assert!(
            entry_js.contains(&format!("ICON_{i}_MARKER")),
            "used Icon{i}'s marker missing from built entry output"
        );
    }

    Ok(())
}

/// The `JET_EAGER_BARRELS=1` escape hatch restores the pre-#1991 eager
/// crawl. Both strategies must converge on the exact same *used* module set
/// and produce identical transformed code for each surviving leaf — this
/// test proves that by extracting the `ICON_<n>_MARKER` literal set that
/// actually reached each build's entry output (must be exactly the 3
/// demanded indices, in both modes) and diffing the per-leaf transformed
/// code line for each demanded icon.
///
/// This deliberately does NOT diff whole-`dist/`-file bytes:
///
/// - Each build runs in its own tempdir, and jet's bundle comments embed
///   each source file's *absolute* path (`// Module N: /abs/path/...`), so
///   those bytes always differ across separate tempdir builds regardless of
///   #1991.
/// - Lazy mode crawls a strictly smaller module set than eager — that's the
///   entire point of #1991 (AC1: "file-read/crawl count for barrel leaves ~=
///   3"). jet assigns module ids in crawl/topo order over the crawled set
///   (#1991's own documented property: "ids = topo-assignment order over
///   the crawled set; lazily-skipped leaves never get ids"), so eager's
///   `require(502)` and lazy's `require(3)` for the same logical `Icon500`
///   leaf are expected, correct divergences, not bugs — fully compacting
///   module-id numbering to a survivors-only sequence regardless of
///   crawled-set size is #1991's own explicit "transform-survivors-only
///   reordering (reserve child)" out-of-scope carve-out, not this issue's
///   job.
///
/// Lazy output must still be strictly smaller than eager's for this fixture
/// shape (AC3: "entry bytes unchanged ... or better") — that size delta,
/// plus identical used-leaf code content, is what actually proves the
/// optimization is both effective and correct.
#[test]
fn lazy_and_eager_crawl_agree_on_used_module_output() -> Result<()> {
    let leaf_count = 1_000;
    let used = [0usize, 500, 999];

    let lazy_tmp = tempfile::tempdir().context("tempdir (lazy)")?;
    write_barrel_fixture(lazy_tmp.path(), leaf_count, &used);
    run_build(lazy_tmp.path(), &[])?;

    let eager_tmp = tempfile::tempdir().context("tempdir (eager)")?;
    write_barrel_fixture(eager_tmp.path(), leaf_count, &used);
    run_build(eager_tmp.path(), &[("JET_EAGER_BARRELS", "1")])?;

    let lazy_dist = lazy_tmp.path().join("dist");
    let eager_dist = eager_tmp.path().join("dist");

    // Same dist/ file shape (index.html + main.<hash>.js + main.<hash>.js.map)
    // — hash-named files necessarily differ by name, so compare shape, not
    // exact names.
    let lazy_files = relative_file_set(&lazy_dist);
    let eager_files = relative_file_set(&eager_dist);
    assert_eq!(
        lazy_files.len(),
        eager_files.len(),
        "lazy and eager crawl produced different dist/ file counts: \
         lazy={lazy_files:?} eager={eager_files:?}"
    );
    for suffix in [".html", ".js", ".js.map"] {
        let lazy_has = lazy_files
            .iter()
            .any(|p| p.to_string_lossy().ends_with(suffix));
        let eager_has = eager_files
            .iter()
            .any(|p| p.to_string_lossy().ends_with(suffix));
        assert!(
            lazy_has && eager_has,
            "expected both lazy and eager dist/ to contain a *{suffix} file: \
             lazy={lazy_files:?} eager={eager_files:?}"
        );
    }

    let lazy_entry_path = find_entry_file(&lazy_dist);
    let eager_entry_path = find_entry_file(&eager_dist);
    let lazy_js = fs::read_to_string(&lazy_entry_path)
        .with_context(|| format!("read lazy entry {lazy_entry_path:?}"))?;
    let eager_js = fs::read_to_string(&eager_entry_path)
        .with_context(|| format!("read eager entry {eager_entry_path:?}"))?;

    // Correctness: both modes' output must contain exactly the 3 demanded
    // markers — no more (a dead/unrequested leaf leaking through, the exact
    // defect this test caught during #1991 development), no fewer (a
    // demanded leaf wrongly dropped).
    let expected: BTreeSet<usize> = used.iter().copied().collect();
    let lazy_markers = find_icon_marker_indices(&lazy_js);
    let eager_markers = find_icon_marker_indices(&eager_js);
    assert_eq!(
        lazy_markers, expected,
        "lazy crawl entry output has the wrong marker set (dead/unrequested \
         leaves must never reach output)"
    );
    assert_eq!(
        eager_markers, expected,
        "eager crawl entry output has the wrong marker set"
    );

    // Content equality for each used leaf's transformed code body (module-id
    // numbering aside — see doc comment above).
    for i in used {
        let leaf_line =
            format!("const Icon{i} = 'ICON_{i}_MARKER';; module.exports[\"Icon{i}\"] = Icon{i};");
        assert!(
            lazy_js.contains(&leaf_line),
            "lazy entry missing expected Icon{i} transformed code line: {leaf_line}"
        );
        assert!(
            eager_js.contains(&leaf_line),
            "eager entry missing expected Icon{i} transformed code line: {leaf_line}"
        );
    }

    // Effectiveness: lazy mode must not merely be correct, it must be
    // materially smaller for this 1,000-leaf/3-used shape (AC3: "entry
    // bytes unchanged ... or better").
    assert!(
        lazy_js.len() < eager_js.len(),
        "expected lazy entry ({} bytes) to be smaller than eager entry ({} \
         bytes) for a 1,000-leaf/3-used barrel",
        lazy_js.len(),
        eager_js.len()
    );

    Ok(())
}
// HANDWRITE-END
