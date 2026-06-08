//! Py3.12 conformance tests for `dict` comprehensions (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_dictcomps.py — basic
//! sections):
//!   construction from a list of key-value pairs, transforming `range`
//!   into a dict, and the `if` filter clause.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_dict_comprehension_from_pair_list() {
    let out = jit_capture(
        r#"print({k: v for k, v in [("a", 1), ("b", 2)]})
"#,
    );
    assert_output(&out, "{'a': 1, 'b': 2}\n");
}

#[test]
fn test_dict_comprehension_transforming_range() {
    let out = jit_capture(
        r#"d = {x: x*x for x in range(4)}
print(sorted(d.items()))
"#,
    );
    assert_output(&out, "[(0, 0), (1, 1), (2, 4), (3, 9)]\n");
}

#[test]
fn test_dict_comprehension_with_filter() {
    let out = jit_capture(
        r#"d = {x: x for x in range(5) if x > 1}
print(sorted(d.items()))
"#,
    );
    assert_output(&out, "[(2, 2), (3, 3), (4, 4)]\n");
}
