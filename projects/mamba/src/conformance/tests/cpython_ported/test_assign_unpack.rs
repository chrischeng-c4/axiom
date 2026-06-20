//! Py3.12 conformance tests for tuple and starred-target assignment
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unpack.py and
//! test_grammar.py — assignment sections):
//!   parallel assignment, swap, list unpack into multiple targets, and
//!   starred targets in any position (front, middle, end).
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_parallel_assignment_and_swap() {
    let out = jit_capture(
        r#"a, b = 1, 2
print(a, b)
a, b = b, a
print(a, b)
"#,
    );
    assert_output(&out, "1 2\n2 1\n");
}

#[test]
fn test_unpack_list_into_named_targets() {
    let out = jit_capture(
        r#"x, y, z = [10, 20, 30]
print(x, y, z)
p, q = (100, 200)
print(p, q)
"#,
    );
    assert_output(&out, "10 20 30\n100 200\n");
}

#[test]
fn test_starred_target_front_middle_end() {
    let out = jit_capture(
        r#"first, *rest = [1, 2, 3, 4, 5]
print(first, rest)
*init, last = [1, 2, 3, 4, 5]
print(init, last)
a, *middle, b = [1, 2, 3, 4, 5]
print(a, middle, b)
"#,
    );
    assert_output(&out, "1 [2, 3, 4, 5]\n[1, 2, 3, 4] 5\n1 [2, 3, 4] 5\n");
}
