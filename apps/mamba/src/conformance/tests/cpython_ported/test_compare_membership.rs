//! Py3.12 conformance tests for chained comparisons, conditional
//! (ternary) expressions, and membership operators (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_compare.py +
//! Lib/test/test_grammar.py — comparison/conditional sections):
//!   chained comparisons, in/not-in across list/str/dict/tuple, ternary
//!   expressions inside list comprehensions.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_chained_numeric_comparisons() {
    let out = jit_capture(
        r#"print(1 < 2 < 3)
print(1 < 5 < 3)
print(1 <= 1 <= 2)
print(3 > 2 > 1)
print(1 == 1 == 1)
x = 5
print(0 < x < 10)
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nTrue\nTrue\nTrue\n");
}

#[test]
fn test_membership_in_and_not_in() {
    let out = jit_capture(
        r#"print(3 in [1, 2, 3])
print(5 in [1, 2, 3])
print("b" in "abc")
print("z" not in "abc")
print(2 in {1: "a", 2: "b"})
print(3 in (1, 2, 3))
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nTrue\nTrue\nTrue\n");
}

#[test]
fn test_ternary_expression_in_comprehension() {
    let out = jit_capture(
        r#"x = 5
print("big" if x > 3 else "small")
nums = [1, 2, 3]
print([n * 10 if n > 1 else n for n in nums])
"#,
    );
    assert_output(&out, "big\n[1, 20, 30]\n");
}
