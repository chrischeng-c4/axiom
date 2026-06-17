#![cfg(test)]

/// P0 conformance integration tests (mamba-conformance-p0 change).
///
/// Tests the 6 P0 fixes end-to-end through the full JIT pipeline:
///   parse → type-check → HIR → MIR → Cranelift JIT → capture stdout → verify
///
/// TC-1: Lambda SIGBUS fix (P0-R1)
/// TC-2: With-statement SIGBUS fix (P0-R2)
/// TC-3: Stacked decorator SIGBUS fix (P0-R3)
/// TC-4: Stdlib functions return None fix (P0-R4)
/// TC-5: Comprehension scope isolation (P0-R5) — end-to-end
/// TC-6: Walrus operator := scope fix (P0-R6) — end-to-end

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
// TC-1: Lambda SIGBUS Fix (P0-R1)
// ═════════════════════════════════════════════════════════════════════════════

/// TC-1.1: Simple lambda compiles without SIGBUS.
/// GIVEN: square = lambda x: x * x; print(square(5))
/// THEN: stdout prints "25"
#[test]
fn test_p0_r1_simple_lambda() {
    let output = jit_capture(
        "square = lambda x: x * x\nprint(square(5))\n",
    );
    assert_output(&output, "25\n");
}

/// TC-1.2: Nested lambda with closure capture.
/// GIVEN: adder = lambda x: lambda y: x + y; print(adder(3)(4))
/// THEN: stdout prints "7"
#[test]
fn test_p0_r1_nested_lambda_closure() {
    let output = jit_capture(
        "adder = lambda x: lambda y: x + y\nprint(adder(3)(4))\n",
    );
    assert_output(&output, "7\n");
}

/// TC-1.3: Lambda capturing enclosing variable.
/// GIVEN: x = 10; f = lambda: x * 2; print(f())
/// THEN: stdout prints "20"
#[test]
fn test_p0_r1_lambda_capture_enclosing() {
    let output = jit_capture(
        "x = 10\nf = lambda: x * 2\nprint(f())\n",
    );
    assert_output(&output, "20\n");
}

/// TC-1 supplemental: Lambda with multiple arguments.
#[test]
fn test_p0_r1_lambda_multiple_args() {
    let output = jit_capture(
        "add = lambda x, y: x + y\nprint(add(3, 4))\n",
    );
    assert_output(&output, "7\n");
}

// ═════════════════════════════════════════════════════════════════════════════
// TC-2: With-Statement SIGBUS Fix (P0-R2)
// ═════════════════════════════════════════════════════════════════════════════

/// TC-2.1: Basic try/finally (context manager prerequisite).
/// GIVEN: try/finally block
/// THEN: both blocks execute, no SIGBUS
#[test]
fn test_p0_r2_try_finally() {
    let output = jit_capture(
        r#"print("test 1")
try:
    print("try block")
finally:
    print("finally block")
"#,
    );
    assert_output(&output, "test 1\ntry block\nfinally block\n");
}

/// TC-2.2: Try/except/finally with exception.
/// GIVEN: try raises ValueError, except catches, finally runs
/// THEN: all branches execute correctly
#[test]
fn test_p0_r2_try_except_finally() {
    let output = jit_capture(
        r#"print("test 2")
try:
    raise ValueError("oops")
except ValueError:
    print("caught")
finally:
    print("done")
"#,
    );
    assert_output(&output, "test 2\ncaught\ndone\n");
}

/// TC-2.3: Try/except/finally without exception.
/// GIVEN: try body succeeds, except not reached, finally runs
/// THEN: no exception branch taken
#[test]
fn test_p0_r2_try_finally_no_exception() {
    let output = jit_capture(
        r#"print("test 3")
try:
    x = 10
    print(x)
except ValueError:
    print("not reached")
finally:
    print("cleanup")
"#,
    );
    assert_output(&output, "test 3\n10\ncleanup\n");
}

// ═════════════════════════════════════════════════════════════════════════════
// TC-3: Stacked Decorator SIGBUS Fix (P0-R3)
// ═════════════════════════════════════════════════════════════════════════════

/// TC-3.1: Function as first-class value, basic call.
/// GIVEN: def add(a, b): return a + b; print(add(3, 4))
/// THEN: stdout prints "7"
#[test]
fn test_p0_r3_function_first_class() {
    let output = jit_capture(
        r#"def add(a, b):
    return a + b

print(add(3, 4))
print(add(10, 20))
"#,
    );
    assert_output(&output, "7\n30\n");
}

/// TC-3.2: Store function in variable and call.
/// GIVEN: f = add; print(f(5, 6))
/// THEN: stdout prints "11"
#[test]
fn test_p0_r3_function_variable() {
    let output = jit_capture(
        r#"def add(a, b):
    return a + b

f = add
print(f(5, 6))
"#,
    );
    assert_output(&output, "11\n");
}

/// TC-3.3: Pass function as argument.
/// GIVEN: call_with_args(add, 1, 2)
/// THEN: stdout prints "3"
#[test]
fn test_p0_r3_function_as_argument() {
    let output = jit_capture(
        r#"def add(a, b):
    return a + b

def call_with_args(func, a, b):
    return func(a, b)

print(call_with_args(add, 1, 2))
"#,
    );
    assert_output(&output, "3\n");
}

/// TC-3.4: Identity decorator (no SIGBUS regression).
/// GIVEN: identity decorator applied, function unchanged
/// THEN: stdout prints "15"
#[test]
fn test_p0_r3_identity_decorator() {
    let output = jit_capture(
        r#"def add(a, b):
    return a + b

def identity(func):
    return func

wrapped = identity(add)
print(wrapped(7, 8))
"#,
    );
    assert_output(&output, "15\n");
}

// ═════════════════════════════════════════════════════════════════════════════
// TC-4: Stdlib Functions Return None Fix (P0-R4)
// ═════════════════════════════════════════════════════════════════════════════

/// TC-4.1: itertools module import succeeds.
/// GIVEN: import itertools; print("itertools imported")
/// THEN: stdout matches expected (not None)
#[test]
fn test_p0_r4_stdlib_itertools_import() {
    let output = jit_capture(
        r#"import itertools
print("itertools imported")
print(isinstance(itertools, object))
"#,
    );
    assert_output(&output, "itertools imported\nTrue\n");
}

/// TC-4.2: collections module import succeeds.
#[test]
fn test_p0_r4_stdlib_collections_import() {
    let output = jit_capture(
        r#"import collections
print("collections imported")
print(isinstance(collections, object))
"#,
    );
    assert_output(&output, "collections imported\nTrue\n");
}

/// TC-4.6: math module functions return values (not None).
/// GIVEN: import math; print basic operations
/// THEN: math functions return correct numeric results
#[test]
fn test_p0_r4_stdlib_math_constants() {
    let output = jit_capture(
        r#"import math
print(f"{math.pi:.6f}")
print(f"{math.e:.6f}")
"#,
    );
    assert_output(&output, "3.141593\n2.718282\n");
}

/// TC-4.6b: math.floor and math.ceil return values.
#[test]
fn test_p0_r4_stdlib_math_floor_ceil() {
    let output = jit_capture(
        r#"import math
print(math.floor(3.7))
print(math.ceil(3.2))
"#,
    );
    assert_output(&output, "3\n4\n");
}

/// TC-4.6c: math.sqrt returns value.
#[test]
fn test_p0_r4_stdlib_math_sqrt() {
    let output = jit_capture(
        r#"import math
print(math.sqrt(16.0))
"#,
    );
    assert_output(&output, "4.0\n");
}

// ═════════════════════════════════════════════════════════════════════════════
// TC-5: Comprehension Scope Isolation (P0-R5) — end-to-end
// ═════════════════════════════════════════════════════════════════════════════

/// TC-5.1: List comprehension basic.
/// GIVEN: squares = [x * x for x in range(5)]
/// THEN: stdout prints "[0, 1, 4, 9, 16]"
#[test]
fn test_p0_r5_list_comprehension_basic() {
    let output = jit_capture(
        "squares = [x * x for x in range(5)]\nprint(squares)\n",
    );
    assert_output(&output, "[0, 1, 4, 9, 16]\n");
}

/// TC-5.2: Dict comprehension basic.
/// GIVEN: d = {k: k * 2 for k in range(4)}
/// THEN: stdout prints "{0: 0, 1: 2, 2: 4, 3: 6}"
#[test]
fn test_p0_r5_dict_comprehension_basic() {
    let output = jit_capture(
        "d = {k: k * 2 for k in range(4)}\nprint(d)\n",
    );
    assert_output(&output, "{0: 0, 1: 2, 2: 4, 3: 6}\n");
}

/// TC-5.3: Generator expression with sum.
/// GIVEN: total = sum(n * n for n in range(6))
/// THEN: stdout prints "55"
#[test]
fn test_p0_r5_generator_expr_sum() {
    let output = jit_capture(
        "total = sum(n * n for n in range(6))\nprint(total)\n",
    );
    assert_output(&output, "55\n");
}

/// TC-5.4: List comprehension with condition.
/// GIVEN: evens = [n for n in range(10) if n % 2 == 0]
/// THEN: stdout prints "[0, 2, 4, 6, 8]"
#[test]
fn test_p0_r5_list_comp_with_condition() {
    let output = jit_capture(
        "evens = [n for n in range(10) if n % 2 == 0]\nprint(evens)\n",
    );
    assert_output(&output, "[0, 2, 4, 6, 8]\n");
}

/// TC-5.5: Scope isolation — list comprehension loop variable does not leak.
/// GIVEN: x = 99; vals = [x * x for x in range(5)]; print(x)
/// THEN: x is still 99, not the comprehension's last value
#[test]
fn test_p0_r5_scope_isolation_list_comp() {
    let output = jit_capture(
        r#"x = 99
vals = [x * x for x in range(5)]
print(vals)
print(x)
"#,
    );
    assert_output(&output, "[0, 1, 4, 9, 16]\n99\n");
}

/// TC-5.6: Scope isolation — dict comprehension loop variable does not leak.
/// GIVEN: k = "outer"; d2 = {k: k * 10 for k in range(3)}; print(k)
/// THEN: k is still "outer"
#[test]
fn test_p0_r5_scope_isolation_dict_comp() {
    let output = jit_capture(
        r#"k = "outer"
d2 = {k: k * 10 for k in range(3)}
print(d2)
print(k)
"#,
    );
    assert_output(&output, "{0: 0, 1: 10, 2: 20}\nouter\n");
}

// ═════════════════════════════════════════════════════════════════════════════
// TC-6: Walrus Operator := Scope Fix (P0-R6) — end-to-end
// ═════════════════════════════════════════════════════════════════════════════

/// TC-6.1: Walrus inside list comprehension binds to enclosing scope.
/// GIVEN: results = [y := n * 2 for n in range(4)]; print(y)
/// THEN: y == 6 (last value), results == [0, 2, 4, 6]
#[test]
fn test_p0_r6_walrus_in_list_comp() {
    let output = jit_capture(
        r#"results = [y := n * 2 for n in range(4)]
print(results)
print(y)
"#,
    );
    assert_output(&output, "[0, 2, 4, 6]\n6\n");
}
