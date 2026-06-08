//! Py3.12 conformance tests for `bin`/`hex`/`oct` and integer `repr`
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_builtin.py — radix
//! conversion sections):
//!   `bin`/`hex`/`oct` formatting with appropriate prefixes, plus
//!   plain `repr` and `str` of int values.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_bin_hex_oct_with_prefixes() {
    let out = jit_capture(
        r#"print(bin(10))
print(hex(255))
print(oct(8))
"#,
    );
    assert_output(&out, "0b1010\n0xff\n0o10\n");
}

#[test]
fn test_bin_hex_oct_on_zero() {
    let out = jit_capture(
        r#"print(bin(0))
print(hex(0))
print(oct(0))
"#,
    );
    assert_output(&out, "0b0\n0x0\n0o0\n");
}

#[test]
fn test_repr_and_str_on_ints() {
    let out = jit_capture(
        r#"print(repr(42))
print(repr(-7))
print(str(1000000))
print(str(0))
"#,
    );
    assert_output(&out, "42\n-7\n1000000\n0\n");
}
