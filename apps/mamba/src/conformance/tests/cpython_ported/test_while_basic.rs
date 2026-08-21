//! Py3.12 conformance tests for basic `while` loop forms
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_grammar.py — while
//! sections): index-driven summation, count-down printing with
//! `end=" "`, and `while True` with `break`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_while_index_summation() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 4, 5]
total = 0
i = 0
while i < len(xs):
    total = total + xs[i]
    i = i + 1
print(total)
"#,
    );
    assert_output(&out, "15\n");
}

#[test]
fn test_while_countdown_with_step() {
    let out = jit_capture(
        r#"i = 10
while i > 0:
    print(i, end=" ")
    i = i - 2
print()
"#,
    );
    assert_output(&out, "10 8 6 4 2 \n");
}

#[test]
fn test_while_true_break() {
    let out = jit_capture(
        r#"n = 0
while True:
    n = n + 1
    if n >= 5:
        break
print(n)
"#,
    );
    assert_output(&out, "5\n");
}
