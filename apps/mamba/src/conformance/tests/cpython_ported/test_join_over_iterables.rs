//! Py3.12 conformance tests for `str.join` over varied iterables
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unicode.py — join
//! sections):
//!   join over list of str, list comprehension, generator expression,
//!   empty iterable, and singleton iterable (separator absent).
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_join_over_list_of_strings() {
    let out = jit_capture(
        r#"print(", ".join(["a", "b", "c"]))
print("-".join(["x"]))
print(" | ".join(["one", "two", "three"]))
"#,
    );
    assert_output(&out, "a, b, c\nx\none | two | three\n");
}

#[test]
fn test_join_over_comprehension_and_generator() {
    let out = jit_capture(
        r#"print("-".join([str(x) for x in range(4)]))
print(" ".join(c for c in "hello"))
"#,
    );
    assert_output(&out, "0-1-2-3\nh e l l o\n");
}

#[test]
fn test_join_on_empty_and_singleton() {
    let out = jit_capture(
        r#"print(repr(",".join([])))
print("x".join(["a"]))
print(repr("-".join([])))
"#,
    );
    assert_output(&out, "''\na\n''\n");
}
