//! Py3.12 conformance tests for set binary operators (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_set.py — binary
//! operator sections): union (`|`), intersection (`&`), difference
//! (`-`), symmetric difference (`^`), and equivalence with the named
//! method counterparts.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_union_and_intersection_operators() {
    let out = jit_capture(
        r#"s = {1, 2, 3, 4, 5}
t = {3, 4, 5, 6, 7}
print(sorted(s | t))
print(sorted(s & t))
"#,
    );
    assert_output(&out, "[1, 2, 3, 4, 5, 6, 7]\n[3, 4, 5]\n");
}

#[test]
fn test_difference_and_symdiff_operators() {
    let out = jit_capture(
        r#"s = {1, 2, 3, 4, 5}
t = {3, 4, 5, 6, 7}
print(sorted(s - t))
print(sorted(t - s))
print(sorted(s ^ t))
"#,
    );
    assert_output(&out, "[1, 2]\n[6, 7]\n[1, 2, 6, 7]\n");
}

#[test]
fn test_operators_match_named_methods() {
    let out = jit_capture(
        r#"s = {1, 2, 3, 4, 5}
t = {3, 4, 5, 6, 7}
print(s.union(t) == s | t)
print(s.intersection(t) == s & t)
print(s.difference(t) == s - t)
print(s.symmetric_difference(t) == s ^ t)
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\nTrue\n");
}
