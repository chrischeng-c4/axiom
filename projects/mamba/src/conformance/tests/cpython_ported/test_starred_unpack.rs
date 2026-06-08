//! Py3.12 conformance tests for starred-target unpacking
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unpack.py —
//! starred-target sections): head/tail starred targets, middle
//! starred targets, and unpacking in a `for` loop with paired
//! tuple elements.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_starred_head_and_tail() {
    let out = jit_capture(
        r#"first, *rest = [1, 2, 3, 4, 5]
print(first)
print(rest)
*head, last = [1, 2, 3, 4, 5]
print(head)
print(last)
"#,
    );
    assert_output(&out, "1\n[2, 3, 4, 5]\n[1, 2, 3, 4]\n5\n");
}

#[test]
fn test_starred_middle_target() {
    let out = jit_capture(
        r#"a, *mid, z = [1, 2, 3, 4, 5]
print(a, mid, z)
"#,
    );
    assert_output(&out, "1 [2, 3, 4] 5\n");
}

#[test]
fn test_tuple_unpack_for_loop_and_swap() {
    let out = jit_capture(
        r#"x, y = 10, 20
x, y = y, x
print(x, y)

pairs = [(1, "a"), (2, "b"), (3, "c")]
for n, s in pairs:
    print(n, s)
"#,
    );
    assert_output(&out, "20 10\n1 a\n2 b\n3 c\n");
}
