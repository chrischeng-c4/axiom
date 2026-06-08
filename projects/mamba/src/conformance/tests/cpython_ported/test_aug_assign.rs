//! Py3.12 conformance tests for augmented assignment, bitwise ops, and
//! multi-target assignment (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_augassign.py +
//! Lib/test/test_bitop.py):
//!   numeric augmented ops, list/str augmented ops, bitwise primitives,
//!   chained simple assignment.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_aug_assign_numeric_chain() {
    let out = jit_capture(
        r#"x = 10
x += 5
print(x)
x -= 3
print(x)
x *= 2
print(x)
x //= 4
print(x)
x **= 2
print(x)
"#,
    );
    assert_output(&out, "15\n12\n24\n6\n36\n");
}

#[test]
fn test_aug_assign_list_and_str() {
    let out = jit_capture(
        r#"y = [1, 2, 3]
y += [4, 5]
print(y)
z = "ab"
z *= 3
print(z)
"#,
    );
    assert_output(&out, "[1, 2, 3, 4, 5]\nababab\n");
}

#[test]
fn test_bitwise_and_or_xor_not() {
    let out = jit_capture(
        r#"print(0b1100 & 0b1010)
print(0b1100 | 0b1010)
print(0b1100 ^ 0b1010)
print(~5)
"#,
    );
    assert_output(&out, "8\n14\n6\n-6\n");
}

#[test]
fn test_bitwise_shift_left_right() {
    let out = jit_capture(
        r#"print(1 << 4)
print(64 >> 2)
print(0b1 << 8)
print(0xff >> 4)
"#,
    );
    assert_output(&out, "16\n16\n256\n15\n");
}

#[test]
fn test_chained_simple_assignment() {
    let out = jit_capture(
        r#"a = b = c = 10
print(a, b, c)
"#,
    );
    assert_output(&out, "10 10 10\n");
}
