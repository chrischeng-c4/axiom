#![cfg(test)]

/// No-arg constructor codegen fix tests (#1109).
///
/// Verifies that `list()`, `tuple()`, `set()`, `dict()` with zero arguments
/// correctly route to `_new` variants instead of `_from_iterable`/`_from_pairs`,
/// and that the one-arg path still works as before.
///
/// Scenarios from spec no-arg-constructor-codegen-fix:
///   S1: list() → empty list
///   S2: tuple() → empty tuple
///   S3: set() → empty set
///   S4: list(range(3)) → [0, 1, 2]
///   S5: tuple([1, 2, 3]) → (1, 2, 3)
///   S6: set([1, 2, 2, 3]) → [1, 2, 3] (sorted)
///   S7: dict() → empty dict

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

const TEST_TIMEOUT_SECS: u64 = 10;

/// Run Python source through the full JIT pipeline, capturing stdout.
fn jit_capture(src: &str) -> String {
    let _jit_guard = JIT_LOCK.lock().unwrap_or_else(|p| p.into_inner());

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

// ═════════════════════════════════════════════════════════════════════════════
// S1: list() with zero args produces empty list (R1, R3, R4)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_s1_list_zero_args_empty_list() {
    let output = jit_capture("x = list()\nprint(x)\n");
    assert_output(&output, "[]\n");
}

#[test]
fn test_s1_list_zero_args_type_name() {
    let output = jit_capture("x = list()\nprint(type(x).__name__)\n");
    assert_output(&output, "list\n");
}

// ═════════════════════════════════════════════════════════════════════════════
// S2: tuple() with zero args produces empty tuple (R1, R3, R4)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_s2_tuple_zero_args_empty_tuple() {
    let output = jit_capture("x = tuple()\nprint(x)\n");
    assert_output(&output, "()\n");
}

#[test]
fn test_s2_tuple_zero_args_len() {
    let output = jit_capture("x = tuple()\nprint(len(x))\n");
    assert_output(&output, "0\n");
}

// ═════════════════════════════════════════════════════════════════════════════
// S3: set() with zero args produces empty set (R1, R3, R4)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_s3_set_zero_args_len() {
    let output = jit_capture("x = set()\nprint(len(x))\n");
    assert_output(&output, "0\n");
}

#[test]
fn test_s3_set_zero_args_type_name() {
    let output = jit_capture("x = set()\nprint(type(x).__name__)\n");
    assert_output(&output, "set\n");
}

// ═════════════════════════════════════════════════════════════════════════════
// S4: list(iterable) still routes to mb_list_from_iterable (R2)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_s4_list_with_range_arg() {
    let output = jit_capture("x = list(range(3))\nprint(x)\n");
    assert_output(&output, "[0, 1, 2]\n");
}

// ═════════════════════════════════════════════════════════════════════════════
// S5: tuple(iterable) still routes to mb_tuple_from_iterable (R2)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_s5_tuple_with_list_arg() {
    let output = jit_capture("x = tuple([1, 2, 3])\nprint(x)\n");
    assert_output(&output, "(1, 2, 3)\n");
}

// ═════════════════════════════════════════════════════════════════════════════
// S6: set(iterable) still routes to mb_set_from_iterable (R2)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_s6_set_with_list_arg_dedup() {
    let output = jit_capture("x = set([1, 2, 2, 3])\nprint(sorted(x))\n");
    assert_output(&output, "[1, 2, 3]\n");
}

// ═════════════════════════════════════════════════════════════════════════════
// S7: dict() with zero args produces empty dict (R1, R4)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_s7_dict_zero_args_empty_dict() {
    let output = jit_capture("d = dict()\nprint(d)\n");
    assert_output(&output, "{}\n");
}

#[test]
fn test_s7_dict_zero_args_len() {
    let output = jit_capture("d = dict()\nprint(len(d))\n");
    assert_output(&output, "0\n");
}
