//! Py3.12 conformance tests for iterating over a `str` (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unicode.py — iteration
//! sections):
//!   `for c in s` yields characters; list-comprehension over a string;
//!   `list(s)` constructor.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_for_loop_iterates_characters() {
    let out = jit_capture(
        r#"out = []
for c in "abc":
    out.append(c)
print(out)
"#,
    );
    assert_output(&out, "['a', 'b', 'c']\n");
}

#[test]
fn test_list_comprehension_over_string() {
    let out = jit_capture(
        r#"print([c for c in "hello"])
print([c.upper() for c in "abc"])
"#,
    );
    assert_output(&out, "['h', 'e', 'l', 'l', 'o']\n['A', 'B', 'C']\n");
}

#[test]
fn test_list_constructor_on_string() {
    let out = jit_capture(
        r#"print(list("hi"))
print(list(""))
print(list("a"))
"#,
    );
    assert_output(&out, "['h', 'i']\n[]\n['a']\n");
}
