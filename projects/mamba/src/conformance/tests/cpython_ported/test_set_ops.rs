//! Py3.12 conformance tests for `set` operator/method equivalents
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_set.py — operator
//! sections):
//!   union (`|`), intersection (`&`), difference (`-`), symmetric
//!   difference (`^`), plus method aliases `.union` / `.intersection`.
//!   Results are sorted into lists for deterministic output.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_set_union_and_intersection_operators() {
    let out = jit_capture(
        r#"a = {1, 2, 3}
b = {2, 3, 4}
print(sorted(a | b))
print(sorted(a & b))
"#,
    );
    assert_output(&out, "[1, 2, 3, 4]\n[2, 3]\n");
}

#[test]
fn test_set_difference_and_symmetric_difference() {
    let out = jit_capture(
        r#"a = {1, 2, 3}
b = {2, 3, 4}
print(sorted(a - b))
print(sorted(a ^ b))
"#,
    );
    assert_output(&out, "[1]\n[1, 4]\n");
}

#[test]
fn test_set_method_aliases_match_operators() {
    let out = jit_capture(
        r#"a = {1, 2, 3}
b = {2, 3, 4}
print(sorted(a.union(b)))
print(sorted(a.intersection(b)))
"#,
    );
    assert_output(&out, "[1, 2, 3, 4]\n[2, 3]\n");
}

#[test]
fn test_set_disjoint_inputs_produce_full_union() {
    let out = jit_capture(
        r#"a = {1, 2}
b = {3, 4}
print(sorted(a | b))
print(sorted(a & b))
"#,
    );
    assert_output(&out, "[1, 2, 3, 4]\n[]\n");
}
