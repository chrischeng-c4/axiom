//! Py3.12 conformance tests for `zip` combined with `dict`/`list` and
//! over heterogeneous iterables (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_builtin.py — zip
//! section):
//!   `dict(zip(...))` building, `list(zip(...))` of tuples, zip over
//!   string iterables, and zip stopping at the shorter input length.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_dict_from_zip() {
    let out = jit_capture(
        r#"keys = ["a", "b", "c"]
vals = [1, 2, 3]
d = dict(zip(keys, vals))
print(sorted(d.items()))
"#,
    );
    assert_output(&out, "[('a', 1), ('b', 2), ('c', 3)]\n");
}

#[test]
fn test_list_of_zip_tuples_and_strings() {
    let out = jit_capture(
        r#"print(list(zip([1, 2, 3], ["a", "b", "c"])))
print(list(zip("abc", "xyz")))
"#,
    );
    assert_output(
        &out,
        "[(1, 'a'), (2, 'b'), (3, 'c')]\n[('a', 'x'), ('b', 'y'), ('c', 'z')]\n",
    );
}

#[test]
fn test_zip_stops_at_shorter() {
    let out = jit_capture(
        r#"print(list(zip([1, 2, 3, 4, 5], ["a", "b"])))
print(list(zip([], [1, 2, 3])))
"#,
    );
    assert_output(&out, "[(1, 'a'), (2, 'b')]\n[]\n");
}
