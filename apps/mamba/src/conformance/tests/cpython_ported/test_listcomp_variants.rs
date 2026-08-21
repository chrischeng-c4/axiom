//! Py3.12 conformance tests for list comprehension variants
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_listcomps.py —
//! basic, filter, and tuple-projection sections):
//!   transform, filter, projection-to-tuple, and arithmetic-on-int
//!   list comprehensions, plus `sum()` over a generator expression.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_transform_and_filter() {
    let out = jit_capture(
        r#"words = ["apple", "bee", "cat", "donkey", "elephant"]
print([len(w) for w in words])
print([w.upper() for w in words])
print([w for w in words if len(w) > 3])
"#,
    );
    assert_output(
        &out,
        "[5, 3, 3, 6, 8]\n['APPLE', 'BEE', 'CAT', 'DONKEY', 'ELEPHANT']\n['apple', 'donkey', 'elephant']\n",
    );
}

#[test]
fn test_projection_to_tuple() {
    let out = jit_capture(
        r#"words = ["apple", "bee", "cat"]
print([(w, len(w)) for w in words])
print([(i, w) for i, w in enumerate(words)])
"#,
    );
    assert_output(
        &out,
        "[('apple', 5), ('bee', 3), ('cat', 3)]\n[(0, 'apple'), (1, 'bee'), (2, 'cat')]\n",
    );
}

#[test]
fn test_arithmetic_filter_and_sum_genexp() {
    let out = jit_capture(
        r#"nums = [1, 2, 3, 4, 5]
print([n * n for n in nums])
print([n for n in nums if n % 2 == 0])
print(sum(n for n in nums))
"#,
    );
    assert_output(&out, "[1, 4, 9, 16, 25]\n[2, 4]\n15\n");
}
