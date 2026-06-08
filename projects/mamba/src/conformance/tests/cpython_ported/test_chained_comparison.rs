//! Py3.12 conformance tests for chained comparison expressions
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_grammar.py — comparison
//! sections):
//!   `a < b < c` is `a<b and b<c`; mixed direction chains, equality
//!   chains, and short-circuit semantics through a variable.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_chained_less_than_in_order() {
    let out = jit_capture(
        r#"print(1 < 2 < 3)
print(1 < 3 < 2)
print(5 > 3 > 1)
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\n");
}

#[test]
fn test_chained_le_and_equality() {
    let out = jit_capture(
        r#"print(1 <= 1 <= 2)
print(1 == 1 == 1)
print(1 != 2 != 3)
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}

#[test]
fn test_chained_range_check_via_variable() {
    let out = jit_capture(
        r#"x = 5
print(0 < x < 10)
print(0 < x < 3)
print(10 > x > 0)
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\n");
}
