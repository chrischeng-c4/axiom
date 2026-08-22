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

fn assert_jit_zero_division_case(code: &str, expected_msg: &str) {
    let res = jit_try(code);
    assert!(res.is_err(), "expected error for: {code:?}");
    let err = res.unwrap_err();
    assert!(
        err.contains("ZeroDivisionError"),
        "expected ZeroDivisionError for {code:?}, got: {err}"
    );
    assert!(
        err.contains(expected_msg),
        "expected msg anchor {expected_msg:?} for {code:?}, got: {err}"
    );

    let control_res = jit_try("print(42)\n");
    assert!(
        control_res.is_ok(),
        "expected clean control execution after zero-division failure, got: {control_res:?}"
    );
    let output = control_res.unwrap();
    assert_eq!(output.trim(), "42");
}

/// Verify JIT integer true division by zero returns ZeroDivisionError and leaves process usable.
#[test]
fn test_jit_zero_division_int_true_div() {
    assert_jit_zero_division_case("x = 1 / 0\n", "division by zero");
}

/// Verify JIT float true division by zero returns ZeroDivisionError and leaves process usable.
#[test]
fn test_jit_zero_division_float_true_div() {
    assert_jit_zero_division_case("x = 1.0 / 0.0\n", "float division by zero");
}

/// Verify JIT integer floor division by zero returns ZeroDivisionError and leaves process usable.
#[test]
fn test_jit_zero_division_int_floor_div() {
    assert_jit_zero_division_case("x = 1 // 0\n", "integer division or modulo by zero");
}

/// Verify JIT float floor division by zero returns ZeroDivisionError and leaves process usable.
#[test]
fn test_jit_zero_division_float_floor_div() {
    assert_jit_zero_division_case("x = 1.0 // 0.0\n", "float floor division by zero");
}

/// Test inconsistent MRO inheritance edge case.
#[test]
fn test_inconsistent_mro_rejected() {
    let src = r#"
class X: pass
class Y: pass
class A(X, Y): pass
class B(Y, X): pass
class C(A, B): pass
"#;
    let res = jit_try(src);
    assert!(res.is_err(), "expected inconsistent MRO to be rejected");
    let err = res.unwrap_err();
    assert!(err.contains("TypeError"), "expected TypeError, got: {err}");
    assert!(
        err.contains("Cannot create a consistent method resolution order (MRO)"),
        "expected MRO message anchor, got: {err}"
    );
}

/// Test that a caught inconsistent class statement does not run its body or bind the rejected name.
#[test]
fn test_inconsistent_mro_caught_exception_and_no_side_effects() {
    let src = r#"
class X: pass
class Y: pass
class A(X, Y): pass
class B(Y, X): pass

body_ran = False
caught = False

try:
    class C(A, B):
        body_ran = True
except TypeError as e:
    caught = "Cannot create a consistent method resolution order (MRO)" in str(e)

bound = "C" in globals() or "C" in locals()
print(f"caught={caught}, body_ran={body_ran}, bound={bound}")
"#;
    let res = jit_try(src);
    assert!(
        res.is_ok(),
        "expected try/except to catch TypeError, got: {res:?}"
    );
    let output = res.unwrap();
    assert_eq!(output.trim(), "caught=True, body_ran=False, bound=False");
}

/// Test that valid diamond and consistent multiple inheritance execute with expected MRO/body behavior.
#[test]
fn test_valid_diamond_and_consistent_multiple_inheritance() {
    let src = r#"
class Base:
    def label(self):
        return "Base"

class Left(Base):
    def label(self):
        return "Left -> " + super().label()

class Right(Base):
    def label(self):
        return "Right -> " + super().label()

class Child(Left, Right):
    def label(self):
        return "Child -> " + super().label()

c = Child()
print(c.label())
"#;
    let res = jit_try(src);
    assert!(
        res.is_ok(),
        "expected valid diamond MRO to execute, got: {res:?}"
    );
    let output = res.unwrap();
    assert_eq!(output.trim(), "Child -> Left -> Right -> Base");
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
    assert!(
        res.is_err(),
        "expected undeclared __slots__ attribute assignment to fail"
    );
    let err = res.unwrap_err();
    assert!(
        err.contains("AttributeError"),
        "expected AttributeError, got: {err}"
    );
    assert!(
        err.contains("Restricted"),
        "expected class anchor 'Restricted', got: {err}"
    );
    assert!(
        err.contains("disallowed"),
        "expected attribute anchor 'disallowed', got: {err}"
    );
}

/// Test positive controls for declared-own slots, inherited slots, and explicit __dict__ slots.
#[test]
fn test_slots_positive_controls() {
    let src = r#"
class Base:
    __slots__ = ('base_slot',)

class Derived(Base):
    __slots__ = ('derived_slot',)

class DictAllowed:
    __slots__ = ('__dict__', 'declared')

b = Base()
b.base_slot = 10

d = Derived()
d.base_slot = 20
d.derived_slot = 30

da = DictAllowed()
da.declared = 40
da.dynamic = 50

print(f"base_slot={b.base_slot}")
print(f"inherited={d.base_slot}, derived={d.derived_slot}")
print(f"declared={da.declared}, dynamic={da.dynamic}")
"#;
    let res = jit_try(src);
    assert!(
        res.is_ok(),
        "expected positive slot controls to execute cleanly, got: {res:?}"
    );
    let output = res.unwrap();
    assert_eq!(
        output.trim(),
        "base_slot=10\ninherited=20, derived=30\ndeclared=40, dynamic=50"
    );
}
