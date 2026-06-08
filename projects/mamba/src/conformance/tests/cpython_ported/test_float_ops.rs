//! Py3.12 conformance tests for float arithmetic and conversion
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_float.py — basic
//! arithmetic, conversion, and round sections): float division and
//! floor division, `round` with ndigits, and conversion between
//! `int`/`float`/`str`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_float_arithmetic_basics() {
    let out = jit_capture(
        r#"x = 3.14
print(x * 2)
print(x / 2)
print(x // 1)
print(2.5 * 4)
print(10.0 / 4)
"#,
    );
    assert_output(&out, "6.28\n1.57\n3.0\n10.0\n2.5\n");
}

#[test]
fn test_round_with_ndigits() {
    let out = jit_capture(
        r#"print(round(3.14159, 1))
print(round(2.567, 2))
print(round(1.5))
print(round(2.5))
print(round(0.1234567, 3))
"#,
    );
    assert_output(&out, "3.1\n2.57\n2\n2\n0.123\n");
}

#[test]
fn test_int_float_str_conversion() {
    let out = jit_capture(
        r#"print(int(3.14))
print(int(-2.7))
print(float(5))
print(float("3.14"))
print(str(2.5))
print(str(0.0))
"#,
    );
    assert_output(&out, "3\n-2\n5.0\n3.14\n2.5\n0.0\n");
}
