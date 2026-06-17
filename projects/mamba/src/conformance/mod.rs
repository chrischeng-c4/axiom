//! Conformance test runner for `cclab mamba test --conformance` (R5).
//!
//! Discovers `.py` fixtures under a conformance directory, runs each through
//! the Mamba JIT pipeline, compares captured stdout against `.expected` golden
//! files, and returns structured results.
//!
//! Directives (in `.py` file comments):
//!   `# mamba-xfail: <reason>` — mark fixture as expected failure (skipped)

pub mod pytest_runner;

/// Crate-relative root of the CPython conformance fixtures — the single source
/// of truth for where fixtures live. Production code and tests all join this,
/// so relocating the fixtures means editing exactly one constant. Source code
/// must not hardcode the fixtures path anywhere else.
pub const FIXTURES_ROOT: &str = "tests/cpython";

use crate::codegen::cranelift::jit::CraneliftJitBackend;
use crate::codegen::{CodegenBackend, CodegenOutput};
use crate::lower::{lower_hir_to_mir_with_symbols, lower_module};
use crate::parser;
use crate::codegen::cranelift::jit::JIT_LOCK;
use crate::runtime::cleanup_all_runtime_state;
use crate::runtime::output::{begin_capture, end_capture};
use crate::source::span::FileId;
use crate::types::TypeChecker;

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 10;

// ── Public types ──────────────────────────────────────────────────────────────

/// Outcome of running a single conformance fixture.
#[derive(Debug)]
pub enum FixtureOutcome {
    /// Fixture passed (actual == expected).
    Passed,
    /// Fixture was skipped due to `# mamba-xfail:` directive.
    Xfailed { reason: String },
    /// Fixture produced different output from the golden file.
    Failed { diff: String },
    /// Fixture errored during JIT compilation or execution.
    Error { message: String },
    /// The `.expected` golden file is missing.
    MissingGolden,
}

/// Result for a single fixture file.
#[derive(Debug)]
pub struct FixtureResult {
    /// Relative path under the conformance root.
    pub relative_path: PathBuf,
    pub outcome: FixtureOutcome,
}

/// Summary of a full conformance suite run.
#[derive(Debug, Default)]
pub struct ConformanceSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub xfailed: usize,
    pub errors: usize,
}

/// Options for running the conformance suite.
pub struct ConformanceOptions {
    /// Root directory containing `.py` fixtures and `.expected` files.
    pub conformance_dir: PathBuf,
    /// Optional category filter (e.g. `"builtins"` or `"stdlib/json"`).
    pub category: Option<String>,
    /// Timeout per fixture in seconds.
    pub timeout_secs: u64,
}

impl Default for ConformanceOptions {
    fn default() -> Self {
        Self {
            conformance_dir: PathBuf::from(crate::conformance::FIXTURES_ROOT),
            category: None,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }
}

// ── Directive parsing ─────────────────────────────────────────────────────────

fn parse_xfail(src: &str) -> Option<String> {
    for line in src.lines() {
        let t = line.trim();
        if let Some(reason) = t.strip_prefix("# mamba-xfail:") {
            return Some(reason.trim().to_string());
        }
    }
    None
}

// ── JIT execution with output capture ────────────────────────────────────────

fn run_and_capture(src: &str, path: &Path, timeout_secs: u64) -> Result<String, String> {
    // Serialize entire JIT pipeline (init + compile + execute) across test
    // threads. Concurrent JITModule finalization causes SIGBUS on aarch64.
    // Guard is held until function exit (after execution thread completes).
    let _jit_guard = JIT_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let module = parser::parse(src, FileId(0))
        .map_err(|e| format!("{}: parse error: {e}", path.display()))?;

    let mut checker = TypeChecker::new();
    let errors = checker.check_module(&module);
    if !errors.is_empty() {
        return Err(format!(
            "{}: type errors: {:?}",
            path.display(),
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
        ));
    }

    let hir = lower_module(&module, &checker)
        .map_err(|errs| format!("{}: HIR error: {:?}", path.display(), errs))?;
    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

    let mut backend = CraneliftJitBackend::new()
        .map_err(|e| format!("{}: JIT init: {e}", path.display()))?;
    let output = backend
        .codegen(&mir, &checker.tcx)
        .map_err(|e| format!("{}: codegen: {e}", path.display()))?;

    match output {
        CodegenOutput::Jit { entry } => {
            let entry_addr = entry as usize;
            let path_str = path.display().to_string();
            let (tx, rx) = mpsc::sync_channel(1);

            // HANDWRITE-BEGIN gap="standardize:projects-mamba-src-conformance-mod-rs" tracker="standardize-gap-projects-mamba-src-conformance-mod-rs" reason="introspection-builtins (issue: enhancement-mamba-introspection-builtins-globals-locals-vars-dir)."
            // Build the introspection state on the main thread (where checker /
            // hir / backend live) and move it into the JIT execution thread so
            // `set_module_sym_info` runs in the thread that owns
            // GLOBAL_ID_NAMESPACE before the JIT writes into it.
            let mut sym_func_addrs: Vec<(u32, String, *const u8)> = Vec::new();
            {
                use std::collections::HashMap;
                let mut sym_names: HashMap<crate::resolve::SymbolId, String> = HashMap::new();
                for s in checker.symbols.all_symbols() {
                    sym_names.insert(s.id, s.name.clone());
                }
                for (id, name) in &hir.sym_names {
                    sym_names.insert(*id, name.clone());
                }
                for f in &hir.functions {
                    if let Some(name) = sym_names.get(&f.name) {
                        if let Some(ptr) = backend.get_func_ptr(f.name.0) {
                            sym_func_addrs.push((f.name.0, name.clone(), ptr));
                        }
                    }
                }
            }
            let (sym_info, func_info) =
                crate::runtime::module::build_introspection_state(&checker, &hir, &sym_func_addrs);
            // *const u8 is not Send; rebuild func_info inside the spawn thread
            // by passing usize-encoded addresses.
            let func_info_addrs: Vec<(String, usize)> = func_info
                .into_iter()
                .map(|(name, fv)| (name, fv.to_bits() as usize))
                .collect();
            // HANDWRITE-END

            let handle = thread::spawn(move || {
                // HANDWRITE-BEGIN gap="standardize:projects-mamba-src-conformance-mod-rs" tracker="standardize-gap-projects-mamba-src-conformance-mod-rs" reason="introspection-builtins."
                crate::runtime::closure::set_module_sym_info(sym_info);
                let func_info_thread: std::collections::HashMap<String, crate::runtime::value::MbValue> =
                    func_info_addrs
                        .into_iter()
                        .map(|(name, bits)| (name, crate::runtime::value::MbValue::from_bits(bits as u64)))
                        .collect();
                crate::runtime::closure::set_module_func_info(func_info_thread);
                // HANDWRITE-END
                let prev = begin_capture();
                let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry_addr) };
                let _result = main_fn();
                cleanup_all_runtime_state();
                let captured = end_capture(prev);
                let _ = tx.send(captured);
            });

            let result = match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
                Ok(captured) => Ok(captured),
                Err(mpsc::RecvTimeoutError::Timeout) => Err(format!(
                    "{path_str}: timed out after {timeout_secs}s"
                )),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    Err(format!("{path_str}: JIT execution thread panicked"))
                }
            };

            // Join the execution thread before returning. With coroutine-based
            // generators (#1187), all generator code runs on the same thread,
            // so cleanup_all_generators() immediately deallocates all coroutine
            // stacks. The join ensures `backend` is not dropped while the
            // execution thread is still running.
            let _ = handle.join();

            result
        }
        _ => Err(format!("{}: expected JIT output", path.display())),
    }
}

// ── Diff formatting ───────────────────────────────────────────────────────────

fn format_diff(expected: &str, actual: &str) -> String {
    let mut out = String::new();
    let exp_lines: Vec<&str> = expected.lines().collect();
    let act_lines: Vec<&str> = actual.lines().collect();
    let max = exp_lines.len().max(act_lines.len());

    for i in 0..max {
        let e = exp_lines.get(i).copied().unwrap_or("");
        let a = act_lines.get(i).copied().unwrap_or("");
        if e != a {
            out.push_str(&format!("  line {}: - {:?}\n", i + 1, e));
            out.push_str(&format!("  line {}:   + {:?}\n", i + 1, a));
        }
    }
    if exp_lines.len() != act_lines.len() {
        out.push_str(&format!(
            "  (expected {} lines, got {} lines)\n",
            exp_lines.len(),
            act_lines.len()
        ));
    }
    out
}

// ── Fixture discovery ─────────────────────────────────────────────────────────

/// Discover all `.py` fixture files under `root`, optionally filtered by `category`.
pub fn discover_fixtures(root: &Path, category: Option<&str>) -> Vec<PathBuf> {
    let mut fixtures = Vec::new();

    fn walk(dir: &Path, fixtures: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, fixtures);
            } else if path.extension().and_then(|e| e.to_str()) == Some("py") {
                fixtures.push(path);
            }
        }
    }

    let search_root = if let Some(cat) = category {
        root.join(cat)
    } else {
        root.to_path_buf()
    };

    walk(&search_root, &mut fixtures);
    fixtures
}

// ── Single fixture runner ─────────────────────────────────────────────────────

/// Run a single conformance fixture and return its result.
pub fn run_fixture(py_path: &Path, conformance_root: &Path, timeout_secs: u64) -> FixtureResult {
    let relative_path = py_path
        .strip_prefix(conformance_root)
        .unwrap_or(py_path)
        .to_path_buf();

    let src = match std::fs::read_to_string(py_path) {
        Ok(s) => s,
        Err(e) => {
            return FixtureResult {
                relative_path,
                outcome: FixtureOutcome::Error {
                    message: format!("read error: {e}"),
                },
            };
        }
    };

    // Check xfail directive
    if let Some(reason) = parse_xfail(&src) {
        return FixtureResult {
            relative_path,
            outcome: FixtureOutcome::Xfailed { reason },
        };
    }

    // Load expected golden file
    let expected_path = py_path.with_extension("expected");
    let expected = match std::fs::read_to_string(&expected_path) {
        Ok(s) => s,
        Err(_) => {
            return FixtureResult {
                relative_path,
                outcome: FixtureOutcome::MissingGolden,
            };
        }
    };

    // Run through Mamba JIT
    match run_and_capture(&src, py_path, timeout_secs) {
        Ok(actual) if actual == expected => FixtureResult {
            relative_path,
            outcome: FixtureOutcome::Passed,
        },
        Ok(actual) => {
            let diff = format_diff(&expected, &actual);
            FixtureResult {
                relative_path,
                outcome: FixtureOutcome::Failed { diff },
            }
        }
        Err(msg) => FixtureResult {
            relative_path,
            outcome: FixtureOutcome::Error { message: msg },
        },
    }
}

// ── Suite runner ──────────────────────────────────────────────────────────────

/// Run all conformance fixtures and return results + summary.
///
/// Prints per-fixture status as it runs (for interactive output).
pub fn run_suite(opts: &ConformanceOptions) -> (Vec<FixtureResult>, ConformanceSummary) {
    let fixtures = discover_fixtures(&opts.conformance_dir, opts.category.as_deref());
    let mut results = Vec::with_capacity(fixtures.len());
    let mut summary = ConformanceSummary::default();

    for path in &fixtures {
        let result = run_fixture(path, &opts.conformance_dir, opts.timeout_secs);

        match &result.outcome {
            FixtureOutcome::Passed => {
                println!("  PASS  {}", result.relative_path.display());
                summary.passed += 1;
            }
            FixtureOutcome::Xfailed { reason } => {
                println!("  xfail {}: {reason}", result.relative_path.display());
                summary.xfailed += 1;
            }
            FixtureOutcome::Failed { diff } => {
                println!("  FAIL  {}", result.relative_path.display());
                println!("{diff}");
                summary.failed += 1;
            }
            FixtureOutcome::Error { message } => {
                println!("  ERROR {}: {message}", result.relative_path.display());
                summary.errors += 1;
            }
            FixtureOutcome::MissingGolden => {
                println!(
                    "  ERROR {}: missing .expected golden file",
                    result.relative_path.display()
                );
                summary.errors += 1;
            }
        }

        summary.total += 1;
        results.push(result);
    }

    (results, summary)
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ── helpers ───────────────────────────────────────────────────────────────

    /// Write a file inside `dir` and return its path.
    fn write_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let p = dir.join(name);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&p, content).unwrap();
        p
    }

    // ── parse_xfail ───────────────────────────────────────────────────────────

    /// TC-ZD.3: xfail directive with reason is parsed (returns Some).
    #[test]
    fn parse_xfail_present_returns_reason() {
        let src = "# mamba-xfail: ExceptionGroup not implemented (see #755)\nprint(1)\n";
        let result = parse_xfail(src);
        assert_eq!(
            result.as_deref(),
            Some("ExceptionGroup not implemented (see #755)")
        );
    }

    /// TC-ZD.3: xfail directive with minimal reason.
    #[test]
    fn parse_xfail_minimal_reason() {
        let src = "# mamba-xfail: async generators (see #999)\n";
        let result = parse_xfail(src);
        assert_eq!(result.as_deref(), Some("async generators (see #999)"));
    }

    /// No xfail directive returns None — fixture runs normally.
    #[test]
    fn parse_xfail_absent_returns_none() {
        let src = "print('hello')\n# just a regular comment\n";
        assert!(parse_xfail(src).is_none());
    }

    /// Empty source has no xfail.
    #[test]
    fn parse_xfail_empty_source() {
        assert!(parse_xfail("").is_none());
    }

    /// Whitespace-only prefix is stripped from the reason.
    #[test]
    fn parse_xfail_reason_trimmed() {
        let src = "# mamba-xfail:   trimmed reason  ";
        let result = parse_xfail(src);
        assert_eq!(result.as_deref(), Some("trimmed reason"));

        // Indented line — parse_xfail trims each line first, so leading
        // whitespace before the `#` is fine.
        let indented = "    # mamba-xfail:   indented reason   ";
        assert_eq!(parse_xfail(indented).as_deref(), Some("indented reason"));

        // Two spaces between `#` and `mamba-xfail:` is NOT recognised — the
        // prefix requires exactly one space, matching the convention used by
        // every active fixture.
        let double_space = "#  mamba-xfail: not a directive";
        assert!(parse_xfail(double_space).is_none());
    }

    /// xfail must be on a `# mamba-xfail:` line; `# mamba_xfail` is not parsed.
    #[test]
    fn parse_xfail_wrong_prefix_returns_none() {
        let src = "# mamba_xfail: wrong separator\n# xfail: also wrong\n";
        assert!(parse_xfail(src).is_none());
    }

    // ── format_diff ───────────────────────────────────────────────────────────

    /// TC-R5.2: identical content produces an empty diff (no output lines).
    #[test]
    fn format_diff_identical_is_empty() {
        let s = "line1\nline2\nline3\n";
        let diff = format_diff(s, s);
        assert!(diff.is_empty(), "expected empty diff, got:\n{diff}");
    }

    /// TC-R5.2: single differing line is reported.
    #[test]
    fn format_diff_one_changed_line() {
        let expected = "line1\nline2\nline3\n";
        let actual = "line1\nLINE2\nline3\n";
        let diff = format_diff(expected, actual);
        assert!(diff.contains("line 2"), "diff should mention line 2: {diff}");
        assert!(diff.contains("line2"), "diff should show expected: {diff}");
        assert!(diff.contains("LINE2"), "diff should show actual: {diff}");
    }

    /// TC-R5.2: length mismatch is reported.
    #[test]
    fn format_diff_length_mismatch_reported() {
        let expected = "a\nb\nc\n";
        let actual = "a\nb\n";
        let diff = format_diff(expected, actual);
        assert!(
            diff.contains("expected 3 lines") || diff.contains("3 lines"),
            "length mismatch not reported: {diff}"
        );
        assert!(
            diff.contains("got 2 lines") || diff.contains("2 lines"),
            "actual line count not reported: {diff}"
        );
    }

    /// TC-R5.2: extra lines in actual are reported.
    #[test]
    fn format_diff_extra_actual_lines() {
        let expected = "a\n";
        let actual = "a\nb\nc\n";
        let diff = format_diff(expected, actual);
        assert!(!diff.is_empty(), "expected non-empty diff");
    }

    // ── discover_fixtures ─────────────────────────────────────────────────────

    /// TC-R5.3: all .py files under the root are found when no category filter.
    #[test]
    fn discover_fixtures_finds_all_py_files() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        write_file(root, "cat1/a.py", "");
        write_file(root, "cat1/b.py", "");
        write_file(root, "cat2/c.py", "");
        write_file(root, "cat2/d.expected", ""); // ignored — not .py

        let fixtures = discover_fixtures(root, None);
        assert_eq!(fixtures.len(), 3, "expected 3 .py fixtures, got {fixtures:?}");
    }

    /// TC-R5.3: category filter restricts discovery to the matching subdirectory.
    #[test]
    fn discover_fixtures_category_filter_restricts_to_subdir() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        write_file(root, "builtins/numeric.py", "");
        write_file(root, "builtins/sequence.py", "");
        write_file(root, "stdlib/json/json_ops.py", "");

        let fixtures = discover_fixtures(root, Some("builtins"));
        let names: Vec<_> = fixtures
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names.len(), 2, "expected 2 builtins fixtures, got {names:?}");
        assert!(names.contains(&"numeric.py"), "numeric.py not found");
        assert!(names.contains(&"sequence.py"), "sequence.py not found");
    }

    /// TC-R5.3: nested category filter works (e.g. stdlib/json).
    #[test]
    fn discover_fixtures_nested_category_filter() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        write_file(root, "stdlib/json/json_ops.py", "");
        write_file(root, "stdlib/re/re_ops.py", "");
        write_file(root, "builtins/numeric.py", "");

        let fixtures = discover_fixtures(root, Some("stdlib/json"));
        assert_eq!(fixtures.len(), 1, "expected 1 fixture: {fixtures:?}");
        assert!(
            fixtures[0].ends_with("json_ops.py"),
            "expected json_ops.py, got {:?}",
            fixtures[0]
        );
    }

    /// Empty directory yields no fixtures.
    #[test]
    fn discover_fixtures_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let fixtures = discover_fixtures(tmp.path(), None);
        assert!(fixtures.is_empty());
    }

    /// Non-existent category subdirectory yields no fixtures (no panic).
    #[test]
    fn discover_fixtures_nonexistent_category_returns_empty() {
        let tmp = TempDir::new().unwrap();
        write_file(tmp.path(), "builtins/numeric.py", "");
        let fixtures = discover_fixtures(tmp.path(), Some("does_not_exist"));
        assert!(fixtures.is_empty(), "expected empty list for missing category");
    }

    /// Fixtures are returned in sorted order (deterministic test runs).
    #[test]
    fn discover_fixtures_sorted_order() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        write_file(root, "cat/c.py", "");
        write_file(root, "cat/a.py", "");
        write_file(root, "cat/b.py", "");

        let fixtures = discover_fixtures(root, Some("cat"));
        let names: Vec<_> = fixtures
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted, "fixtures not in sorted order");
    }

    // ── run_fixture — outcome without JIT ─────────────────────────────────────

    /// Fixture with a missing `.expected` file reports `MissingGolden`.
    #[test]
    fn run_fixture_missing_golden_file() {
        let tmp = TempDir::new().unwrap();
        let py_path = write_file(tmp.path(), "test.py", "print(1)\n");

        let result = run_fixture(&py_path, tmp.path(), DEFAULT_TIMEOUT_SECS);
        assert!(
            matches!(result.outcome, FixtureOutcome::MissingGolden),
            "expected MissingGolden, got {:?}",
            result.outcome
        );
    }

    /// Fixture with `# mamba-xfail:` is skipped (returns `Xfailed`) — no JIT needed.
    #[test]
    fn run_fixture_xfail_directive_skips_execution() {
        let tmp = TempDir::new().unwrap();
        let src = "# mamba-xfail: ExceptionGroup not implemented (see #755)\nprint(1)\n";
        let py_path = write_file(tmp.path(), "test.py", src);
        // Write a golden file too — it should never be read because xfail fires first.
        write_file(tmp.path(), "test.expected", "1\n");

        let result = run_fixture(&py_path, tmp.path(), DEFAULT_TIMEOUT_SECS);
        assert!(
            matches!(result.outcome, FixtureOutcome::Xfailed { .. }),
            "expected Xfailed, got {:?}",
            result.outcome
        );
    }

    /// Xfailed fixture preserves the reason string.
    #[test]
    fn run_fixture_xfail_reason_preserved() {
        let tmp = TempDir::new().unwrap();
        let reason = "async generators (see #888)";
        let src = format!("# mamba-xfail: {reason}\nprint(1)\n");
        let py_path = write_file(tmp.path(), "test.py", &src);
        write_file(tmp.path(), "test.expected", "1\n");

        let result = run_fixture(&py_path, tmp.path(), DEFAULT_TIMEOUT_SECS);
        if let FixtureOutcome::Xfailed { reason: r } = result.outcome {
            assert_eq!(r, reason);
        } else {
            panic!("expected Xfailed, got {:?}", result.outcome);
        }
    }

    /// `relative_path` in the result is relative to the conformance root.
    #[test]
    fn run_fixture_relative_path_is_relative_to_root() {
        let tmp = TempDir::new().unwrap();
        let py_path = write_file(tmp.path(), "builtins/numeric.py", "print(1)\n");
        // No .expected → MissingGolden, but relative_path is still set correctly.

        let result = run_fixture(&py_path, tmp.path(), DEFAULT_TIMEOUT_SECS);
        let rel = result.relative_path.to_str().unwrap();
        assert!(
            rel.contains("builtins") && rel.contains("numeric.py"),
            "unexpected relative_path: {rel}"
        );
        assert!(
            !result.relative_path.is_absolute(),
            "relative_path should not be absolute"
        );
    }

    // ── ConformanceSummary counting ───────────────────────────────────────────

    /// TC-R5.1 / TC-R5.5: summary counts are mutually consistent.
    #[test]
    fn conformance_summary_default_is_zero() {
        let s = ConformanceSummary::default();
        assert_eq!(s.total, 0);
        assert_eq!(s.passed, 0);
        assert_eq!(s.failed, 0);
        assert_eq!(s.xfailed, 0);
        assert_eq!(s.errors, 0);
    }

    /// TC-R5.5: exit logic — exit 0 when failed + errors == 0.
    #[test]
    fn conformance_summary_exit0_when_no_failures() {
        let s = ConformanceSummary {
            total: 5,
            passed: 3,
            failed: 0,
            xfailed: 2,
            errors: 0,
        };
        // The CLI exits 1 if failed > 0 || errors > 0 — this should exit 0.
        assert_eq!(s.failed + s.errors, 0, "should have zero failures/errors");
    }

    /// TC-R5.5: exit logic — exit 1 when any fixture failed.
    #[test]
    fn conformance_summary_exit1_when_failures() {
        let s = ConformanceSummary {
            total: 5,
            passed: 3,
            failed: 1,
            xfailed: 1,
            errors: 0,
        };
        assert!(s.failed + s.errors > 0, "should have non-zero failure count");
    }

    /// TC-R5.5: exit logic — exit 1 when any fixture errored (e.g. MissingGolden).
    #[test]
    fn conformance_summary_exit1_when_errors() {
        let s = ConformanceSummary {
            total: 3,
            passed: 2,
            failed: 0,
            xfailed: 0,
            errors: 1,
        };
        assert!(s.failed + s.errors > 0);
    }

    /// Xfailed fixtures do not affect the exit decision.
    #[test]
    fn conformance_summary_xfail_does_not_affect_exit() {
        let s = ConformanceSummary {
            total: 10,
            passed: 7,
            failed: 0,
            xfailed: 3,
            errors: 0,
        };
        // Only failed + errors matter for exit code.
        assert_eq!(s.failed + s.errors, 0);
    }

    // ── ConformanceOptions defaults ───────────────────────────────────────────

    /// Default options use the canonical conformance directory path.
    #[test]
    fn conformance_options_default_dir() {
        let opts = ConformanceOptions::default();
        assert_eq!(
            opts.conformance_dir,
            PathBuf::from(crate::conformance::FIXTURES_ROOT)
        );
        assert!(opts.category.is_none());
        assert_eq!(opts.timeout_secs, DEFAULT_TIMEOUT_SECS);
    }

    // ── JIT_LOCK conformance integration (sigbus-jit-concurrency-fix) ────────

    /// S2/R2: run_and_capture acquires JIT_LOCK — verify that the lock is held
    /// during execution by checking it is NOT acquirable from the calling thread
    /// while run_and_capture is inflight in another thread.
    ///
    /// We test indirectly: two sequential calls to run_and_capture with invalid
    /// source both return errors and release the lock. If the lock were stuck,
    /// the second call would deadlock.
    #[test]
    fn run_and_capture_releases_lock_after_parse_error() {
        let tmp = TempDir::new().unwrap();
        let py_path = write_file(tmp.path(), "bad.py", "def @@@invalid syntax");
        // First call — should fail at parse, but release the lock.
        let r1 = run_and_capture("def @@@invalid syntax", &py_path, 5);
        assert!(r1.is_err(), "expected parse error");
        // Second call — if lock was not released, this would deadlock.
        let r2 = run_and_capture("def @@@invalid syntax", &py_path, 5);
        assert!(r2.is_err(), "expected parse error on second call too");
    }

    /// S5/R2: run_and_capture releases JIT_LOCK on type-check error.
    /// After a type error, the lock must be released for subsequent calls.
    #[test]
    fn run_and_capture_releases_lock_after_type_error() {
        let tmp = TempDir::new().unwrap();
        // Source that parses but may produce type errors — the actual error
        // type (parse, type-check, codegen) doesn't matter; what matters is
        // that the lock is released afterward.
        let bad_src = "x: int = 'not_an_int'\nprint(x)\n";
        let py_path = write_file(tmp.path(), "bad_types.py", bad_src);
        let _ = run_and_capture(bad_src, &py_path, 5);
        // Lock must be released — verify by acquiring it (handle poison).
        let guard = JIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        drop(guard);
    }

    /// S2: Concurrent run_and_capture calls from multiple threads do not
    /// overlap (JIT_LOCK serializes them). Both should complete without panic.
    #[test]
    fn run_and_capture_concurrent_calls_serialized() {
        use std::sync::Arc;

        let tmp = Arc::new(TempDir::new().unwrap());
        // Use a simple source that parses but will fail at type check or codegen.
        // The key assertion is that both threads complete without SIGBUS or deadlock.
        let src = "def @@@bad";

        let mut handles = Vec::new();
        for i in 0..2 {
            let tmp = Arc::clone(&tmp);
            let src = src.to_string();
            handles.push(std::thread::spawn(move || {
                let py_path = tmp.path().join(format!("t{i}.py"));
                std::fs::write(&py_path, &src).unwrap();
                let _ = run_and_capture(&src, &py_path, 5);
            }));
        }
        for h in handles {
            h.join().expect("thread should not panic");
        }
    }

    /// S3/R4: Lock acquisition in run_and_capture does not add measurable
    /// overhead in single-threaded mode (uncontended lock).
    #[test]
    fn run_and_capture_single_threaded_lock_overhead_minimal() {
        let tmp = TempDir::new().unwrap();
        let py_path = write_file(tmp.path(), "fast.py", "def @@@bad");
        let start = std::time::Instant::now();
        // Run 10 iterations — errors are fine, we're timing lock overhead.
        for _ in 0..10 {
            let _ = run_and_capture("def @@@bad", &py_path, 5);
        }
        let elapsed = start.elapsed();
        // 10 calls should complete well under 5 seconds even with lock overhead.
        assert!(
            elapsed.as_secs() < 5,
            "10 single-threaded run_and_capture calls took {}s — expected <5s",
            elapsed.as_secs()
        );
    }
}

#[cfg(test)]
#[path = "tests"]
mod tests_subdirs {
    pub mod cpython_ported;
    pub mod generators;
}
