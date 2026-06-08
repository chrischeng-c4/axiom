//! Py3.12 conformance tests for tuple operations (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_tuple.py — arithmetic
//! and membership sections): indexing/length/slicing, reductions
//! (`sum`/`min`/`max`), `in` membership, and concat/repeat operators.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_tuple_index_len_slice() {
    let out = jit_capture(
        r#"t = (1, 2, 3, 4, 5)
print(t)
print(t[0], t[-1])
print(len(t))
print(t[1:4])
"#,
    );
    assert_output(&out, "(1, 2, 3, 4, 5)\n1 5\n5\n(2, 3, 4)\n");
}

#[test]
fn test_tuple_reductions_and_membership() {
    let out = jit_capture(
        r#"t = (1, 2, 3, 4, 5)
print(sum(t))
print(min(t), max(t))
print(2 in t)
print(99 in t)
print(t.count(3))
"#,
    );
    assert_output(&out, "15\n1 5\nTrue\nFalse\n1\n");
}

#[test]
fn test_tuple_concat_and_repeat() {
    let out = jit_capture(
        r#"t = (1, 2, 3)
print(t + (4, 5))
print(t * 2)
print((0,) + t)
print(() + t)
print(("x",) * 4)
"#,
    );
    assert_output(
        &out,
        "(1, 2, 3, 4, 5)\n(1, 2, 3, 1, 2, 3)\n(0, 1, 2, 3)\n(1, 2, 3)\n('x', 'x', 'x', 'x')\n",
    );
}
