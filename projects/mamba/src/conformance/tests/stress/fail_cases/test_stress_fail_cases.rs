//! Expected failure and crash-probing test cases (#gen12_fuzzing).
//!
//! Uses `catch_unwind` and `should_panic` patterns to verify error returns
//! and graceful failure handling for invalid code.

use super::super::jit_try;
use crate::parser;
use crate::source::span::FileId;

/// Verify that malformed syntax fails gracefully with Err.
#[test]
fn test_syntax_error_returns_err() {
    let invalid_inputs = [
        "def (:\n",
        "class :\n",
        "if else:\n",
        "x = 1; if True: pass\n",
        "try: pass except:\n",
    ];

    for src in invalid_inputs {
        let res = parser::parse(src, FileId(0));
        assert!(res.is_err(), "expected parse error for: {src:?}");
    }
}

/// Verify that runtime errors in JIT execution (ZeroDivisionError, KeyError, IndexError)
/// return an Error or panic result safely without crashing the host process.
#[test]
fn test_jit_runtime_errors_handled_safely() {
    let zero_div = "x = 1 / 0\n";
    let res = jit_try(zero_div);
    assert!(res.is_err(), "expected error for division by zero");

    let index_err = "items = []\nprint(items[0])\n";
    let res_idx = jit_try(index_err);
    assert!(res_idx.is_err(), "expected error for index out of bounds");

    let key_err = "d = {}\nprint(d['missing'])\n";
    let res_key = jit_try(key_err);
    assert!(res_key.is_err(), "expected error for missing key in dict");
}

/// Test inconsistent MRO inheritance edge case.
#[test]
fn test_inconsistent_mro_rejected() {
    let src = r#"
class A: pass
class B(A): pass
class C(A, B): pass
"#;
    let res = jit_try(src);
    // Inconsistent MRO should be rejected by type checker or runtime
    assert!(res.is_err(), "expected inconsistent MRO to be rejected");
}

/// Test undeclared __slots__ attribute assignment failure.
#[test]
fn test_slots_undeclared_attribute_rejected() {
    let src = r#"
class Restricted:
    __slots__ = ('allowed',)

r = Restricted()
r.allowed = 1
r.disallowed = 2
"#;
    let res = jit_try(src);
    assert!(res.is_err(), "expected undeclared __slots__ attribute assignment to fail");
}
