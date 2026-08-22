//! Test harness helpers for cpython_ported integration test.
//! CPython 3.12 ported conformance tests for Mamba (#759).
//!
//! Each submodule corresponds to a CPython test file under `Lib/test/`.
//! Tests are ported from the CPython 3.12.0 tag (commit a6cb7e5d45):
//!   https://github.com/python/cpython/tree/v3.12.0
//!
//! The helper `jit_capture` and `assert_output` mirror the pattern from
//! `tests/cpython/core/generators/mod.rs` — kept self-contained to avoid
//! cross-binary dependencies.
//!
//! Run with:
//!   cargo test -p mamba --test conformance_set cpython_ported::
//!
//! @issue #759

#![allow(dead_code)]

use mamba::codegen::cranelift::jit::{CraneliftJitBackend, JIT_LOCK};
use mamba::codegen::{CodegenBackend, CodegenOutput};
use mamba::lower::{lower_hir_to_mir_with_symbols, lower_module};
use mamba::parser;
use mamba::runtime::cleanup_all_runtime_state;
use mamba::runtime::exception;
use mamba::runtime::output::{begin_capture, end_capture};
use mamba::source::span::FileId;
use mamba::types::TypeChecker;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// 20s: hang detection only. Slowest known legitimate test (set_methods
// test_gc family, ~9.3s solo) needs ~2x headroom against shared-box CPU
// contention; 10s converted contention into phantom timeouts (#2529 r3).
const TEST_TIMEOUT_SECS: u64 = 20;

/// Monotonic counter used to build a unique per-execution cwd-sandbox
/// directory name (combined with the process id). See
/// `jit_capture_with_exception`'s sandbox setup below (#2529 r3).
static SANDBOX_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Private helper: run JIT pipeline returning (captured_stdout, pending_exception_string).
fn jit_capture_with_exception(src: &str) -> (String, Option<String>) {
    let _jit_guard = JIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // #2585 E2: record the outer test thread's name (libtest names each
    // test's thread after its fully-qualified path) so a `debug_validate_obj`
    // violation on the *spawned* JIT worker thread below — which does not
    // inherit that name and previously panicked as `thread '<unnamed>'`
    // with no attribution — can be traced back to the triggering test.
    // JIT_LOCK above serializes execution to at most one in flight per
    // process, so this process-global breadcrumb cannot be clobbered by a
    // concurrently-running test.
    #[cfg(debug_assertions)]
    mamba::runtime::rc::set_current_test_name(
        std::thread::current().name().unwrap_or("<unnamed test thread>"),
    );

    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let errors = checker.check_module(&module);
    if !errors.is_empty() {
        panic!(
            "type errors: {:?}",
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
        );
    }

    let (tx, rx) = mpsc::sync_channel(1);

    let handle = thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let hir = match lower_module(&module, &checker) {
                Ok(hir) => hir,
                Err(_) => {
                    let _ = tx.send(Err("HIR lowering failed".to_string()));
                    return;
                }
            };
            let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

            let mut backend = match CraneliftJitBackend::new() {
                Ok(b) => b,
                Err(e) => {
                    let _ = tx.send(Err(format!("JIT init failed: {e}")));
                    return;
                }
            };
            let output = match backend.codegen(&mir, &checker.tcx) {
                Ok(o) => o,
                Err(e) => {
                    let _ = tx.send(Err(format!("JIT codegen failed: {e}")));
                    return;
                }
            };

            let CodegenOutput::Jit { entry } = output else {
                let _ = tx.send(Err("expected JIT output".to_string()));
                return;
            };

            // cwd-sandbox (#2529 r3): some embedded fixtures write PID-named
            // scratch files (`@mamba_test_<pid>`) into the process cwd
            // without fully cleaning up after themselves. Since JIT_LOCK
            // serializes execution within this process but not cleanup
            // ordering across tests, those artifacts can collide with a
            // later test that reuses the same PID-derived name. Give every
            // execution its own unique temp cwd so such writes can never
            // collide, regardless of shard/thread composition. Safe because
            // the entire execution (this closure) runs while `_jit_guard`
            // (JIT_LOCK) is held, so only one execution is ever using the
            // process-global cwd at a time.
            //
            // The sandbox is nested *inside* the crate's own `target/` tree
            // (via `CARGO_MANIFEST_DIR`, a compile-time constant, not an
            // OS-wide temp dir) rather than `std::env::temp_dir()`: some
            // stdlib shims (e.g. `test.test_ast` resolution in
            // `src/runtime/stdlib/test_mod.rs::oracle_test_package_dir`)
            // locate the vendored CPython oracle package by walking up
            // ancestors of `std::env::current_dir()` looking for
            // `tests/cpython/.cache/oracle-env/...` under the crate root.
            // Chdir-ing outside the crate tree (e.g. into `/tmp`) orphans
            // that lookup and breaks `import test.test_ast`-style fixtures.
            // Nesting under `<CARGO_MANIFEST_DIR>/target/jit_sandbox/` keeps
            // the crate root as an ancestor of the sandboxed cwd, so that
            // lookup still succeeds, while each execution still gets its
            // own unique leaf directory so `@mamba_test_*` writes can never
            // collide.
            let orig_cwd = std::env::current_dir().ok();
            let sandbox_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("jit_sandbox")
                .join(format!(
                    "mamba_jit_{}_{}",
                    std::process::id(),
                    SANDBOX_COUNTER.fetch_add(1, Ordering::Relaxed)
                ));
            let sandbox_ready = std::fs::create_dir_all(&sandbox_dir).is_ok()
                && std::env::set_current_dir(&sandbox_dir).is_ok();

            let prev = begin_capture();
            let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry) };
            let panicked =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| main_fn())).is_err();

            let pending_exception = exception::mb_catch_exception();
            let pending = if pending_exception.is_none() {
                exception::mb_take_uncaught_traceback()
            } else {
                let exc_type = exception::get_exception_type_pub(pending_exception)
                    .unwrap_or_else(|| "Exception".to_string());
                let message = exception::get_exception_message_pub(pending_exception)
                    .unwrap_or_default();
                Some(if message.is_empty() {
                    exc_type
                } else {
                    format!("{exc_type}: {message}")
                })
            };

            cleanup_all_runtime_state();
            let captured = end_capture(prev);

            // Best-effort: restore original cwd and clean up the sandbox.
            // Never panic here -- a cleanup failure must not turn a passing
            // test into a spurious failure.
            if sandbox_ready {
                if let Some(cwd) = orig_cwd.as_ref() {
                    let _ = std::env::set_current_dir(cwd);
                }
            }
            let _ = std::fs::remove_dir_all(&sandbox_dir);

            let _ = tx.send(Ok((captured, pending, panicked)));
        })
        .expect("spawn JIT worker");

    let (captured, pending) = match rx.recv_timeout(Duration::from_secs(TEST_TIMEOUT_SECS)) {
        Ok(Ok((captured, pending, panicked))) => {
            if panicked {
                panic!("JIT execution panicked; captured output:\n{captured}");
            }
            (captured, pending)
        }
        Ok(Err(msg)) => panic!("{msg}"),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("JIT execution timed out after {TEST_TIMEOUT_SECS}s");
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("JIT execution thread panicked");
        }
    };

    let _ = handle.join();
    (captured, pending)
}

/// Run Python source through the full JIT pipeline, capturing stdout.
/// Acquires JIT_LOCK to serialize across concurrent test threads.
pub fn jit_capture(src: &str) -> String {
    let (captured, pending) = jit_capture_with_exception(src);
    if let Some(tb) = pending {
        panic!("uncaught Python exception: {tb}\ncaptured output:\n{captured}");
    }
    captured
}

/// Assert captured stdout equals `expected` (trailing newlines ignored).
pub fn assert_output(actual: &str, expected: &str) {
    let actual_trimmed = actual.trim_end();
    let expected_trimmed = expected.trim_end();
    if actual_trimmed != expected_trimmed {
        let a_lines: Vec<&str> = actual_trimmed.lines().collect();
        let e_lines: Vec<&str> = expected_trimmed.lines().collect();
        let max = a_lines.len().max(e_lines.len());
        let mut diff = String::new();
        for i in 0..max {
            let a = a_lines.get(i).copied().unwrap_or("<missing>");
            let e = e_lines.get(i).copied().unwrap_or("<missing>");
            if a != e {
                diff.push_str(&format!(
                    "  line {}: expected {:?}, got {:?}\n",
                    i + 1,
                    e,
                    a
                ));
            }
        }
        panic!(
            "output mismatch:\n--- expected ---\n{expected_trimmed}\n--- actual ---\n{actual_trimmed}\n--- diff ---\n{diff}"
        );
    }
}

/// Assert that captured stdout contains a substring.
pub fn assert_contains(actual: &str, needle: &str) {
    if !actual.contains(needle) {
        panic!("output missing substring {needle:?}:\n--- actual ---\n{actual}");
    }
}

/// Directives parsed from type wall `.py` fixture comments.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct TypeWallDirectives {
    /// Expected error type from `# mamba-strict-type: <Type>`
    pub strict_type: Option<String>,
    /// Xfail reason from `# mamba-xfail: <reason>`
    pub xfail: Option<String>,
}

/// Parse type wall directives from fixture source lines.
pub fn parse_type_wall_directives(src: &str) -> TypeWallDirectives {
    let mut directives = TypeWallDirectives::default();
    for line in src.lines() {
        let t = line.trim();
        if let Some(val) = t.strip_prefix("# mamba-strict-type:") {
            directives.strict_type = Some(val.trim().to_string());
        } else if let Some(val) = t.strip_prefix("# mamba-xfail:") {
            directives.xfail = Some(val.trim().to_string());
        }
    }
    directives
}

/// Run a type wall fixture source string and return captured output or directive status.
///
/// - `# mamba-xfail: <reason>` short-circuits to `"XFAIL: <reason>"` without
///   running anything.
/// - `# mamba-strict-type: <Type>` checks static type checker rejection first.
///   If statically rejected -> returns `"STRICT_TYPE_REJECTED"`.
///   If accepted by checker -> falls back to JIT execution. If the pending
///   exception matches `<Type>` -> returns `"RUNTIME_REJECTED: <Type>"`.
///   Otherwise panics with "type wall breach".
/// - No directive: runs the full JIT pipeline via [`jit_capture`].
pub fn run_type_wall_fixture(src: &str) -> String {
    let directives = parse_type_wall_directives(src);
    if let Some(reason) = directives.xfail {
        return format!("XFAIL: {reason}");
    }
    if let Some(ty) = directives.strict_type {
        let module = parser::parse(src, FileId(0)).expect("parse failed");
        let mut checker = TypeChecker::new();
        let errors = checker.check_module(&module);
        if !errors.is_empty() {
            return "STRICT_TYPE_REJECTED".to_string();
        }

        // Checker accepted -> fallback to JIT execution runtime check
        let (_captured, pending) = jit_capture_with_exception(src);
        if let Some(exc) = pending {
            let exc_type = exc.split(':').next().unwrap_or("").trim();
            if exc_type == ty {
                return format!("RUNTIME_REJECTED: {exc_type}");
            }
            panic!(
                "type wall breach: expected static rejection ({ty}), got runtime exception {exc}"
            );
        }
        panic!(
            "type wall breach: expected static rejection ({ty}), type checker accepted and runtime executed cleanly"
        );
    }
    jit_capture(src)
}

#[cfg(test)]
mod type_wall_tests {
    use super::*;

    #[test]
    fn test_parse_type_wall_directives_strict_type() {
        let src = "# mamba-strict-type: TypeError\nprint('hello')\n";
        let dirs = parse_type_wall_directives(src);
        assert_eq!(dirs.strict_type.as_deref(), Some("TypeError"));
        assert_eq!(dirs.xfail, None);
    }

    #[test]
    fn test_parse_type_wall_directives_xfail() {
        let src = "# mamba-xfail: pending type wall fix\nprint('hello')\n";
        let dirs = parse_type_wall_directives(src);
        assert_eq!(dirs.strict_type, None);
        assert_eq!(dirs.xfail.as_deref(), Some("pending type wall fix"));
    }

    #[test]
    fn test_run_type_wall_fixture_xfail() {
        let src = "# mamba-xfail: pending fix\n";
        let out = run_type_wall_fixture(src);
        assert_eq!(out, "XFAIL: pending fix");
    }

    /// A genuinely wrong-typed snippet (positional-only `int` parameter
    /// called with a `str`, mirroring
    /// `tests/cpython/type/core/arg_annotation/positional_only_int_arg_called_with_str.py`)
    /// must be statically rejected by the type checker without ever
    /// reaching the JIT.
    #[test]
    fn test_run_type_wall_fixture_strict_type_rejected() {
        let src = "# mamba-strict-type: TypeError\ndef requires_count(count: int, /) -> int:\n    return count\n\n\nrequires_count(\"3\")\n";
        let out = run_type_wall_fixture(src);
        assert_eq!(out, "STRICT_TYPE_REJECTED");
    }

    /// Legal, well-typed code paired with a `# mamba-strict-type:` directive
    /// is a wall *breach*: the checker accepted source the fixture claims
    /// should be statically rejected. That must panic loudly, not silently
    /// pass.
    #[test]
    #[should_panic(expected = "type wall breach")]
    fn test_run_type_wall_fixture_breach_panics() {
        let src = "# mamba-strict-type: TypeError\nprint('hello')\n";
        run_type_wall_fixture(src);
    }

    /// A snippet that passes static type checking but raises the expected
    /// exception at runtime must return `"RUNTIME_REJECTED: <ExcType>"`.
    #[test]
    fn test_run_type_wall_fixture_runtime_rejected() {
        let src = "# mamba-strict-type: ZeroDivisionError\nx = 1 / 0\n";
        let out = run_type_wall_fixture(src);
        assert_eq!(out, "RUNTIME_REJECTED: ZeroDivisionError");
    }
}

#[cfg(test)]
mod jit_capture_signal_tests {
    use super::*;

    /// Harness self-test (#2428): an uncaught Python exception must surface
    /// as a failed Rust test via a panic, not silently disappear into empty
    /// captured stdout.
    #[test]
    #[should_panic(expected = "uncaught Python exception")]
    fn test_jit_capture_signals_uncaught_exception() {
        jit_capture("raise ValueError(\"boom\")\n");
    }
}



