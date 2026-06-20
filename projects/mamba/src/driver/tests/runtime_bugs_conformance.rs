#![cfg(test)]

/// Runtime bugs conformance integration tests (mamba-runtime-bugs change).
///
/// Tests the 5 bug fixes end-to-end through the full JIT pipeline:
///   parse -> type-check -> HIR -> MIR -> Cranelift JIT -> capture stdout -> verify
///
/// T1: Semicolon statement separator (R1)
/// T2: ZeroDivisionError on floor division by zero (R2)
/// T3: Decorator return value propagation (R3)
/// T4: Nested f-string evaluation (R4)
/// T5: json.dumps return value (R5)
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

/// Assert that parsing a given source fails (returns Err).
fn assert_parse_error(src: &str) {
    let result = parser::parse(src, FileId(0));
    assert!(
        result.is_err(),
        "expected parse error, but parsing succeeded"
    );
}

// =============================================================================
// T1: Semicolon Statement Separator (R1)
// =============================================================================

/// TC-1.1: Two assignments separated by semicolons.
/// GIVEN: a = 1; b = 2; print(a); print(b)
/// THEN: stdout is "1\n2\n"
#[test]
fn test_r1_semicolon_two_assignments() {
    let output = jit_capture("a = 1; b = 2; print(a); print(b)\n");
    assert_output(&output, "1\n2\n");
}

/// TC-1.2: Print and assignment separated by semicolons.
/// GIVEN: print(1); x = 2; print(x)
/// THEN: stdout is "1\n2\n"
#[test]
fn test_r1_semicolon_print_and_assign() {
    let output = jit_capture("print(1); x = 2; print(x)\n");
    assert_output(&output, "1\n2\n");
}

/// TC-1.3: Trailing semicolon is tolerated.
/// GIVEN: x = 1; print(x);
/// THEN: no parse error, stdout is "1\n"
#[test]
fn test_r1_semicolon_trailing() {
    let output = jit_capture("x = 1; print(x);\n");
    assert_output(&output, "1\n");
}

/// TC-1.4: Multiple consecutive semicolons (empty statements).
/// GIVEN: a = 1;; b = 2; print(a); print(b)
/// THEN: stdout is "1\n2\n"
#[test]
fn test_r1_semicolon_consecutive() {
    let output = jit_capture("a = 1;; b = 2; print(a); print(b)\n");
    assert_output(&output, "1\n2\n");
}

/// TC-1.5: Compound statement after semicolon is a parse error.
/// GIVEN: x = 1; if True:\n    pass
/// THEN: parse error (compound statements not allowed after `;`)
#[test]
fn test_r1_semicolon_compound_parse_error() {
    assert_parse_error("x = 1; if True:\n    pass\n");
}

/// TC-1.6: Three statements on one line.
/// GIVEN: a = 1; b = 2; c = a + b; print(c)
/// THEN: stdout is "3\n"
#[test]
fn test_r1_semicolon_three_statements() {
    let output = jit_capture("a = 1; b = 2; c = a + b; print(c)\n");
    assert_output(&output, "3\n");
}

// =============================================================================
// T2: ZeroDivisionError on Floor Division by Zero (R2)
// =============================================================================

/// TC-2.1: Integer floor division by zero raises ZeroDivisionError.
/// GIVEN: try: x = 10 // 0 except ZeroDivisionError: print("caught int")
/// THEN: stdout contains "caught int"
#[test]
fn test_r2_floor_div_zero_int() {
    let output = jit_capture(
        r#"try:
    x = 10 // 0
except ZeroDivisionError:
    print("caught int")
"#,
    );
    assert_output(&output, "caught int\n");
}

/// TC-2.2: Float floor division by zero raises ZeroDivisionError.
/// GIVEN: try: x = 10.0 // 0.0 except ZeroDivisionError: print("caught float")
/// THEN: stdout is "caught float\n"
#[test]
fn test_r2_floor_div_zero_float() {
    let output = jit_capture(
        r#"try:
    x = 10.0 // 0.0
except ZeroDivisionError:
    print("caught float")
"#,
    );
    assert_output(&output, "caught float\n");
}

/// TC-2.3: Normal floor division unchanged (non-regression).
/// GIVEN: print(7 // 2)
/// THEN: stdout is "3\n"
#[test]
fn test_r2_floor_div_normal() {
    let output = jit_capture("print(7 // 2)\n");
    assert_output(&output, "3\n");
}

/// TC-2.4: Negative floor division unchanged (non-regression).
/// GIVEN: print(-7 // 2)
/// THEN: stdout is "-4\n"
#[test]
fn test_r2_floor_div_negative() {
    let output = jit_capture("print(-7 // 2)\n");
    assert_output(&output, "-4\n");
}

// =============================================================================
// T3: Decorator Return Value Propagation (R3)
// =============================================================================

/// TC-3.1: Simple decorator preserves return value with multi-arg call.
/// GIVEN: @double def add(a, b): return a + b; print(add(3, 4))
/// THEN: stdout is "14\n" (not None)
#[test]
fn test_r3_decorator_return_multi_arg() {
    let output = jit_capture(
        r#"def double(f):
    def wrapper(a, b):
        return f(a, b) * 2
    return wrapper

@double
def add(a, b):
    return a + b

print(add(3, 4))
"#,
    );
    assert_output(&output, "14\n");
}

/// TC-3.2: Identity decorator returns original value.
/// GIVEN: @identity def greet(): return 42; print(greet())
/// THEN: stdout is "42\n"
#[test]
fn test_r3_decorator_identity() {
    let output = jit_capture(
        r#"def identity(f):
    return f

@identity
def greet():
    return 42

print(greet())
"#,
    );
    assert_output(&output, "42\n");
}

/// TC-3.3: Stacked decorators preserve return chain.
/// GIVEN: @add_one @double2 def val(): return 5; print(val())
/// THEN: stdout is "11\n" (add_one(double2(val))() = (5*2)+1)
/// NOTE: Currently returns 0 due to closure capture across stacked decorators.
/// This is a separate issue from the decorator return value fix (#1084).
#[test]
fn test_r3_decorator_stacked() {
    let output = jit_capture(
        r#"def add_one(f):
    def w():
        return f() + 1
    return w

def double2(f):
    def w():
        return f() * 2
    return w

@add_one
@double2
def val():
    return 5

print(val())
"#,
    );
    assert_output(&output, "11\n");
}

// =============================================================================
// T4: Nested F-String Evaluation (R4)
// =============================================================================

/// TC-4.1: Simple nested f-string with literal.
/// GIVEN: print(f"{f'{42}'}")
/// THEN: stdout is "42\n"
#[test]
fn test_r4_nested_fstring_literal() {
    let output = jit_capture("print(f\"{f'{42}'}\")\n");
    assert_output(&output, "42\n");
}

/// TC-4.2: Nested f-string with variable.
/// GIVEN: x = 5; print(f"{f'{x}'}")
/// THEN: stdout is "5\n"
#[test]
fn test_r4_nested_fstring_variable() {
    let output = jit_capture("x = 5\nprint(f\"{f'{x}'}\")\n");
    assert_output(&output, "5\n");
}

/// TC-4.4: Nested f-string with expression.
/// GIVEN: print(f"{f'{1 + 2}'}")
/// THEN: stdout is "3\n"
#[test]
fn test_r4_nested_fstring_expression() {
    let output = jit_capture("print(f\"{f'{1 + 2}'}\")\n");
    assert_output(&output, "3\n");
}

/// TC-4.5: Three-level nested f-string.
/// GIVEN: print(f"a{f"b{f"c"}"}")
/// THEN: stdout is "abc\n"
#[test]
fn test_r4_nested_fstring_three_level() {
    let output = jit_capture("print(f\"a{f\"b{f\"c\"}\"}\")  \n");
    assert_output(&output, "abc\n");
}

/// TC-4.6: Non-nested f-string (regression guard).
/// GIVEN: x = 10; print(f"val={x}")
/// THEN: stdout is "val=10\n"
#[test]
fn test_r4_fstring_non_nested_regression() {
    let output = jit_capture("x = 10\nprint(f\"val={x}\")\n");
    assert_output(&output, "val=10\n");
}

// =============================================================================
// T5: json.dumps Return Value (R5)
// =============================================================================

/// TC-5.2: json.dumps with list.
/// GIVEN: import json; print(json.dumps([1, 2, 3]))
/// THEN: stdout is "[1, 2, 3]\n"
#[test]
fn test_r5_json_dumps_list() {
    let output = jit_capture(
        r#"import json
print(json.dumps([1, 2, 3]))
"#,
    );
    assert_output(&output, "[1, 2, 3]\n");
}

/// TC-5.3: json.dumps with string.
/// GIVEN: import json; print(json.dumps("hello"))
/// THEN: stdout is '"hello"\n'
#[test]
fn test_r5_json_dumps_string() {
    let output = jit_capture(
        r#"import json
print(json.dumps("hello"))
"#,
    );
    assert_output(&output, "\"hello\"\n");
}

/// TC-5.4: json.dumps with None.
/// GIVEN: import json; print(json.dumps(None))
/// THEN: stdout is "null\n"
#[test]
fn test_r5_json_dumps_none() {
    let output = jit_capture(
        r#"import json
print(json.dumps(None))
"#,
    );
    assert_output(&output, "null\n");
}

/// TC-5.6: json.dumps result used in expression (non-regression).
/// GIVEN: import json; s = json.dumps([1]); print(len(s))
/// THEN: stdout is "3\n" (len("[1]") == 3)
#[test]
fn test_r5_json_dumps_result_usable() {
    let output = jit_capture(
        r#"import json
s = json.dumps([1])
print(len(s))
"#,
    );
    assert_output(&output, "3\n");
}
