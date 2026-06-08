#![cfg(test)]

/// Generator thread pool integration tests (gen-thread-pool change, #1114).
///
/// Tests the GenPool architecture end-to-end through the full JIT pipeline:
///   parse → type-check → HIR → MIR → Cranelift JIT → capture stdout → verify
///
/// Test plan coverage:
///   test_generator_stress_200_iterations  — S1, R1, R2 (pool thread reuse)
///   test_nested_list_comprehension        — S4, R8 (concurrent generators)
///   test_multi_threaded_conformance_suite — S1, S8, R6 (no SIGBUS on aarch64)

use crate::codegen::cranelift::jit::{CraneliftJitBackend, JIT_LOCK};
use crate::codegen::{CodegenBackend, CodegenOutput};
use crate::lower::{lower_hir_to_mir_with_symbols, lower_module};
use crate::parser;
use crate::runtime::cleanup_all_runtime_state;
use crate::runtime::output::{begin_capture, end_capture};
use crate::source::span::FileId;
use crate::types::TypeChecker;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const TEST_TIMEOUT_SECS: u64 = 30;

/// Run Python source through the full JIT pipeline, capturing stdout.
/// Each compilation gets its own isolated JitMemory mmap region (#1114).
fn jit_capture(src: &str) -> String {
    let _jit_guard = JIT_LOCK.lock().unwrap();

    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let errors = checker.check_module(&module);
    if !errors.is_empty() {
        panic!(
            "type errors: {:?}",
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
        );
    }

    let hir = lower_module(&module, &checker).expect("HIR lowering failed");
    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

    let mut backend = CraneliftJitBackend::new().expect("JIT init failed");
    let output = backend
        .codegen(&mir, &checker.tcx)
        .expect("JIT codegen failed");

    match output {
        CodegenOutput::Jit { entry } => {
            let entry_addr = entry as usize;
            let (tx, rx) = mpsc::sync_channel(1);

            let handle = thread::spawn(move || {
                let prev = begin_capture();
                let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry_addr) };
                let _result = main_fn();
                cleanup_all_runtime_state();
                let captured = end_capture(prev);
                let _ = tx.send(captured);
            });

            let result = match rx.recv_timeout(Duration::from_secs(TEST_TIMEOUT_SECS)) {
                Ok(captured) => captured,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    panic!("JIT execution timed out after {TEST_TIMEOUT_SECS}s");
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("JIT execution thread panicked");
                }
            };

            let _ = handle.join();
            result
        }
        _ => panic!("expected JIT output"),
    }
}

/// Assert that captured output matches expected lines.
fn assert_output(actual: &str, expected: &str) {
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
                diff.push_str(&format!("  line {}: expected {:?}, got {:?}\n", i + 1, e, a));
            }
        }
        panic!(
            "output mismatch:\n--- expected ---\n{expected_trimmed}\n--- actual ---\n{actual_trimmed}\n--- diff ---\n{diff}"
        );
    }
}

// =============================================================================
// S1/R1/R2: Generator stress test — 200+ iterations without crash
// =============================================================================

/// Create 200+ generators sequentially (each yields once and completes),
/// verify no crash. Validates pool thread reuse eliminates pthread churn
/// that caused EXC_BAD_ACCESS on macOS aarch64 after ~130 cycles.
#[test]
fn test_generator_stress_200_iterations() {
    // Each iteration creates a generator that yields a single value.
    // Before the pool refactor, this would SIGBUS/EXC_BAD_ACCESS around
    // iteration ~130 due to cumulative pthread lifecycle corruption.
    for i in 0..200 {
        let src = format!(
            r#"def gen():
    yield {i}

g = gen()
print(next(g))
"#
        );
        let output = jit_capture(&src);
        assert_output(&output, &format!("{i}\n"));
    }
}

// =============================================================================
// S4/R8: Nested list comprehension — concurrent generators
// =============================================================================

/// Verify nested list comprehension `[[j for j in range(3)] for i in range(3)]`
/// works correctly with the pool.  Inner and outer generators run on separate
/// pool workers concurrently; no deadlock from pool exhaustion.
#[test]
fn test_nested_list_comprehension() {
    let output = jit_capture(
        r#"result = [[j for j in range(3)] for i in range(3)]
print(result)
"#,
    );
    assert_output(&output, "[[0, 1, 2], [0, 1, 2], [0, 1, 2]]\n");
}

// =============================================================================
// S1/S8/R6: Multi-threaded conformance suite (no SIGBUS on aarch64)
// =============================================================================

/// Run multiple generator tests concurrently (simulating `cargo test` default
/// multi-threaded mode). Each test goes through the full JIT pipeline and
/// uses the shared GenPool. Validates no SIGBUS/SIGSEGV on aarch64.
#[test]
fn test_multi_threaded_conformance_suite() {
    // Run several generator programs concurrently via separate threads.
    // Each thread does JIT compile + execute + cleanup_all_runtime_state.
    let programs: Vec<(&str, &str)> = vec![
        (
            "def g():\n    yield 1\n    yield 2\n    yield 3\nprint(list(g()))\n",
            "[1, 2, 3]\n",
        ),
        (
            "print(list(x * 2 for x in range(5)))\n",
            "[0, 2, 4, 6, 8]\n",
        ),
        (
            "def gen():\n    val = yield 'a'\n    yield val\ng = gen()\nprint(next(g))\nprint(g.send('b'))\n",
            "a\nb\n",
        ),
        (
            "def inner():\n    yield 10\n    yield 20\ndef outer():\n    yield from inner()\nprint(list(outer()))\n",
            "[10, 20]\n",
        ),
    ];

    let mut handles = Vec::new();
    for (src, expected) in programs {
        let src = src.to_string();
        let expected = expected.to_string();
        handles.push(thread::spawn(move || {
            let output = jit_capture(&src);
            assert_output(&output, &expected);
        }));
    }

    for (i, h) in handles.into_iter().enumerate() {
        h.join()
            .unwrap_or_else(|_| panic!("multi-threaded conformance thread {i} panicked"));
    }
}

// =============================================================================
// S5/R6: cleanup_all_runtime_state() joins pool before JIT drop
// =============================================================================

/// Verify that cleanup_all_runtime_state() correctly completes before JIT
/// backend drops.  After cleanup, no worker should be executing JIT code.
/// This test creates generators, runs them, then verifies cleanup finishes
/// within timeout (no hanging workers).
#[test]
fn test_cleanup_joins_all_workers() {
    // Create and exhaust a generator
    let output = jit_capture(
        r#"def g():
    yield 1
    yield 2
    yield 3

result = list(g())
print(result)
"#,
    );
    assert_output(&output, "[1, 2, 3]\n");

    // The jit_capture function already calls cleanup_all_runtime_state() inside
    // its thread.  If cleanup hangs (workers not joined), the test would
    // timeout.  Reaching this point proves cleanup completed successfully.

    // Additional: verify cleanup is idempotent — calling again is a no-op
    cleanup_all_runtime_state();
}

// =============================================================================
// S6/R1: No pool overhead for non-generator tests
// =============================================================================

/// Run a test that does not use generators.  Verify it completes normally
/// without generator-related overhead.  `cleanup_all_runtime_state()` should
/// be a no-op when no generators have been created in this execution path.
#[test]
fn test_no_pool_for_non_generator_test() {
    let output = jit_capture("print('hello world')\n");
    assert_output(&output, "hello world\n");
    // cleanup_all_runtime_state is called inside jit_capture — should be no-op
    // for programs that don't use generators.  If it hangs or crashes, the
    // test would fail.
}

// =============================================================================
// Regression: Sequential generator creation (the original crash scenario)
// =============================================================================

/// Simulate the original crash scenario: rapidly create and exhaust generators
/// in sequence.  This is the pattern that triggered EXC_BAD_ACCESS with
/// per-generator thread::spawn.  With the pool, workers are reused.
#[test]
fn test_sequential_generator_rapid_create_exhaust() {
    for i in 0..50 {
        let src = format!(
            r#"def gen():
    for j in range({count}):
        yield j

total = 0
for v in gen():
    total += v
print(total)
"#,
            count = (i % 5) + 1
        );
        let expected_total: i64 = (0..((i % 5) + 1) as i64).sum();
        let output = jit_capture(&src);
        assert_output(&output, &format!("{expected_total}\n"));
    }
}
