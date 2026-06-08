//! Py3.12 conformance tests for basic `tuple` behavior (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_tuple.py — basic
//! sections):
//!   indexing (including negative), `len`, `+`/`*`, `in` membership,
//!   lexicographic comparison, constructor, and empty/singleton repr.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_tuple_indexing_and_len() {
    let out = jit_capture(
        r#"t = (1, 2, 3)
print(t[0])
print(t[-1])
print(t[1])
print(len(t))
"#,
    );
    assert_output(&out, "1\n3\n2\n3\n");
}

#[test]
fn test_tuple_concat_repeat_and_membership() {
    let out = jit_capture(
        r#"print((1, 2) + (3, 4))
print((1,) * 3)
print(1 in (1, 2, 3))
print(4 in (1, 2, 3))
"#,
    );
    assert_output(&out, "(1, 2, 3, 4)\n(1, 1, 1)\nTrue\nFalse\n");
}

#[test]
fn test_tuple_comparison_lexicographic() {
    let out = jit_capture(
        r#"print((1, 2) == (1, 2))
print((1, 2) < (1, 3))
print((1, 2) < (2, 0))
print((1, 2, 3) > (1, 2))
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\nTrue\n");
}

#[test]
fn test_tuple_constructor_and_singleton_empty() {
    let out = jit_capture(
        r#"print(tuple([1, 2, 3]))
print(tuple())
print((42,))
print(())
"#,
    );
    assert_output(&out, "(1, 2, 3)\n()\n(42,)\n()\n");
}
