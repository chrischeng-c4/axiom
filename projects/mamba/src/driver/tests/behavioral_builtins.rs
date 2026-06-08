#![cfg(test)]

/// Py3.12 behavioral conformance: builtin function tests (T1-T5).
///
/// Part of py312-behavioral-conformance spec (mamba-conformance-p0 change):
///   T1: Numeric builtin edge cases (R1)
///   T2: Type introspection edge cases (R2)
///   T3: String/repr builtins (R3)
///   T4: Collection builtins (R4)
///   T5: Print kwargs (R5)
///
/// Each test runs Python source through the full JIT pipeline:
///   parse -> type-check -> HIR -> MIR -> Cranelift JIT -> capture stdout -> verify
///
/// Tests marked `#[ignore]` require features not yet implemented (tracked as xfail
/// in the fixture-based harness). Remove `#[ignore]` as features land.

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

// ═══════════════════════════════════════════════════════════════════════════════
// T1: Numeric Builtin Edge Cases (R1)
// ═══════════════════════════════════════════════════════════════════════════════

/// T1.1: abs(-0.0) == 0.0.
#[test]
fn test_t1_1_abs_negative_zero() {
    let output = jit_capture("print(abs(-0.0))\n");
    assert_output(&output, "0.0\n");
}

/// T1.2: round(2.5) == 2 (banker's rounding).
#[test]
fn test_t1_2_round_bankers_even() {
    let output = jit_capture("print(round(2.5))\n");
    assert_output(&output, "2\n");
}

/// T1.3: round(3.5) == 4 (banker's rounding).
#[test]
fn test_t1_3_round_bankers_odd() {
    let output = jit_capture("print(round(3.5))\n");
    assert_output(&output, "4\n");
}

/// T1.4: divmod(7, 3) == (2, 1).
#[test]
fn test_t1_4_divmod() {
    let output = jit_capture("print(divmod(7, 3))\n");
    assert_output(&output, "(2, 1)\n");
}

/// T1.5: pow(2, -1) == 0.5.
#[test]
fn test_t1_5_pow_negative_exponent() {
    let output = jit_capture("print(pow(2, -1))\n");
    assert_output(&output, "0.5\n");
}

/// T1.6: pow(2, 10, 1000) == 24 (modular exponentiation).
#[test]
fn test_t1_6_pow_modular() {
    let output = jit_capture("print(pow(2, 10, 1000))\n");
    assert_output(&output, "24\n");
}

/// T1.7: int('0xff', 16) == 255.
#[test]
fn test_t1_7_int_with_base() {
    let output = jit_capture("print(int('0xff', 16))\n");
    assert_output(&output, "255\n");
}

/// T1.8: float('inf') == inf.
#[test]
fn test_t1_8_float_inf() {
    let output = jit_capture("print(float('inf'))\n");
    assert_output(&output, "inf\n");
}

/// T1 supplemental: bin/hex/oct formatting.
#[test]
fn test_t1_bin_hex_oct() {
    let output = jit_capture(
        r#"print(bin(255))
print(hex(255))
print(oct(255))
print(bin(-1))
print(hex(-1))
"#,
    );
    assert_output(&output, "0b11111111\n0xff\n0o377\n-0b1\n-0x1\n");
}

/// T1 supplemental: pow(2, 10) basic (no modulus).
#[test]
fn test_t1_pow_basic() {
    let output = jit_capture("print(2 ** 10)\n");
    assert_output(&output, "1024\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T2: Type Introspection Edge Cases (R2)
// ═══════════════════════════════════════════════════════════════════════════════

/// T2.1: isinstance(True, int) == True — bool is subclass of int.
#[test]
fn test_t2_1_isinstance_bool_int() {
    let output = jit_capture("print(isinstance(True, int))\n");
    assert_output(&output, "True\n");
}

/// T2.2: isinstance(1, (str, float, int)) == True — tuple of types.
#[test]
fn test_t2_2_isinstance_tuple_of_types() {
    let output = jit_capture("print(isinstance(1, (str, float, int)))\n");
    assert_output(&output, "True\n");
}

/// T2.3: issubclass(bool, int) == True.
#[test]
fn test_t2_3_issubclass_bool_int() {
    let output = jit_capture("print(issubclass(bool, int))\n");
    assert_output(&output, "True\n");
}

/// T2.4: getattr(obj, 'missing', 'default') returns default.
#[test]
fn test_t2_4_getattr_default() {
    let output = jit_capture("print(getattr(object(), 'missing', 'default'))\n");
    assert_output(&output, "default\n");
}

/// T2.5: hasattr(object(), 'x') == False.
#[test]
fn test_t2_5_hasattr_false() {
    let output = jit_capture("print(hasattr(object(), 'x'))\n");
    assert_output(&output, "False\n");
}

/// T2.6: callable(len) == True.
#[test]
fn test_t2_6_callable_true() {
    let output = jit_capture("print(callable(len))\n");
    assert_output(&output, "True\n");
}

/// T2.7: callable(42) == False.
#[test]
fn test_t2_7_callable_false() {
    let output = jit_capture("print(callable(42))\n");
    assert_output(&output, "False\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T3: String/Repr Builtins (R3)
// ═══════════════════════════════════════════════════════════════════════════════

/// T3.1: repr('hello') == "'hello'".
#[test]
fn test_t3_1_repr_string() {
    let output = jit_capture("print(repr('hello'))\n");
    assert_output(&output, "'hello'\n");
}

/// T3.2: repr([1, 2, 3]) == '[1, 2, 3]'.
#[test]
fn test_t3_2_repr_list() {
    let output = jit_capture("print(repr([1, 2, 3]))\n");
    assert_output(&output, "[1, 2, 3]\n");
}

/// T3.3: chr(8364) == '€'.
#[test]
fn test_t3_3_chr_unicode() {
    let output = jit_capture("print(chr(8364))\n");
    assert_output(&output, "€\n");
}

/// T3.4: ord('€') == 8364.
#[test]
fn test_t3_4_ord_unicode() {
    let output = jit_capture("print(ord('€'))\n");
    assert_output(&output, "8364\n");
}

/// T3.5: format(3.14159, '.2f') == '3.14'.
#[test]
fn test_t3_5_format_float() {
    let output = jit_capture("print(format(3.14159, '.2f'))\n");
    assert_output(&output, "3.14\n");
}

/// T3.6: format(42, '08b') == '00101010'.
#[test]
fn test_t3_6_format_binary() {
    let output = jit_capture("print(format(42, '08b'))\n");
    assert_output(&output, "00101010\n");
}

/// T3 supplemental: repr(42) == '42'.
#[test]
fn test_t3_repr_int() {
    let output = jit_capture("print(repr(42))\n");
    assert_output(&output, "42\n");
}

/// T3 supplemental: repr(None) == 'None'.
#[test]
fn test_t3_repr_none() {
    let output = jit_capture("print(repr(None))\n");
    assert_output(&output, "None\n");
}

/// T3 supplemental: repr(True) == 'True'.
#[test]
fn test_t3_repr_bool() {
    let output = jit_capture("print(repr(True))\n");
    assert_output(&output, "True\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T4: Collection Builtins — sorted/min/max (R4)
// ═══════════════════════════════════════════════════════════════════════════════

/// T4.1: sorted('hello') == ['e', 'h', 'l', 'l', 'o'].
#[test]
fn test_t4_1_sorted_string() {
    let output = jit_capture("print(sorted('hello'))\n");
    assert_output(&output, "['e', 'h', 'l', 'l', 'o']\n");
}

/// T4.2: sorted([3,1,2], reverse=True) == [3, 2, 1].
#[test]
fn test_t4_2_sorted_reverse() {
    let output = jit_capture("print(sorted([3, 1, 2], reverse=True))\n");
    assert_output(&output, "[3, 2, 1]\n");
}

/// T4.3: sorted(['bb','a','ccc'], key=len) == ['a', 'bb', 'ccc'].
#[test]
fn test_t4_3_sorted_key() {
    let output = jit_capture("print(sorted(['bb', 'a', 'ccc'], key=len))\n");
    assert_output(&output, "['a', 'bb', 'ccc']\n");
}

/// T4.4: min('hello') == 'e'.
#[test]
fn test_t4_4_min_string() {
    let output = jit_capture("print(min('hello'))\n");
    assert_output(&output, "e\n");
}

/// T4.5: max([], default='empty') == 'empty'.
#[test]
fn test_t4_5_max_default() {
    let output = jit_capture("print(max([], default='empty'))\n");
    assert_output(&output, "empty\n");
}

/// T4.6: all([]) == True (vacuous truth).
#[test]
fn test_t4_6_all_empty() {
    let output = jit_capture("print(all([]))\n");
    assert_output(&output, "True\n");
}

/// T4.7: any([]) == False.
#[test]
fn test_t4_7_any_empty() {
    let output = jit_capture("print(any([]))\n");
    assert_output(&output, "False\n");
}

/// T4.8: sum([1.5, 2.5], start=10) == 14.0.
#[test]
fn test_t4_8_sum_with_start() {
    let output = jit_capture("print(sum([1.5, 2.5], start=10))\n");
    assert_output(&output, "14.0\n");
}

/// T4 supplemental: sorted basic int list.
#[test]
fn test_t4_sorted_basic() {
    let output = jit_capture("print(sorted([3, 1, 4, 1, 5]))\n");
    assert_output(&output, "[1, 1, 3, 4, 5]\n");
}

/// T4 supplemental: all/any with boolean lists.
#[test]
fn test_t4_all_any_booleans() {
    let output = jit_capture(
        r#"print(all([True, True, True]))
print(all([True, False, True]))
print(any([False, False, True]))
"#,
    );
    assert_output(&output, "True\nFalse\nTrue\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T5: Print with sep/end kwargs (R5)
// ═══════════════════════════════════════════════════════════════════════════════

/// T5.1: print(1, 2, 3, sep='-') => "1-2-3".
#[test]
fn test_t5_1_print_sep() {
    let output = jit_capture("print(1, 2, 3, sep='-')\n");
    assert_output(&output, "1-2-3\n");
}

/// T5.2: print('hello', end='!!!\\n') => "hello!!!".
#[test]
fn test_t5_2_print_end() {
    let output = jit_capture("print('hello', end='!!!\\n')\n");
    assert_output(&output, "hello!!!\n");
}

/// T5.3: print('a', 'b', sep='', end='') => "ab".
#[test]
fn test_t5_3_print_sep_end_empty() {
    let output = jit_capture("print('a', 'b', sep='', end='')\n");
    assert_output(&output, "ab");
}
