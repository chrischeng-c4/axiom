//! Py3.12 conformance tests for integer bitwise operators (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_int.py — bitwise
//! sections):
//!   `&`, `|`, `^`, `~`, `<<`, `>>` over decimal and `0b`/`0x` literals.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_bitwise_and_or_xor() {
    let out = jit_capture(
        r#"print(5 & 3)
print(5 | 3)
print(5 ^ 3)
"#,
    );
    assert_output(&out, "1\n7\n6\n");
}

#[test]
fn test_bitwise_not_negates_plus_one() {
    let out = jit_capture(
        r#"print(~5)
print(~0)
print(~-1)
"#,
    );
    assert_output(&out, "-6\n-1\n0\n");
}

#[test]
fn test_bitwise_shifts() {
    let out = jit_capture(
        r#"print(1 << 4)
print(16 >> 2)
print(1 << 0)
print(255 >> 4)
"#,
    );
    assert_output(&out, "16\n4\n1\n15\n");
}

#[test]
fn test_bitwise_with_binary_and_hex_literals() {
    let out = jit_capture(
        r#"print(0b1010 | 0b0101)
print(0xff & 0x0f)
print(0b1100 ^ 0b1010)
"#,
    );
    assert_output(&out, "15\n15\n6\n");
}
