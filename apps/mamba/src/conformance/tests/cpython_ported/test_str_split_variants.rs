//! Py3.12 conformance tests for `str.split` variants (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unicode.py — split
//! sections):
//!   default-whitespace split, `maxsplit` from left, `rsplit` from
//!   right, `splitlines`, and consecutive-delimiter behavior.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_str_default_whitespace_split() {
    let out = jit_capture(
        r#"print("a b  c".split())
print("  hi  ".split())
print("one two three".split())
"#,
    );
    assert_output(&out, "['a', 'b', 'c']\n['hi']\n['one', 'two', 'three']\n");
}

#[test]
fn test_str_split_and_rsplit_with_maxsplit() {
    let out = jit_capture(
        r#"print("a,b,c,d".split(",", 2))
print("a,b,c,d".rsplit(",", 2))
"#,
    );
    assert_output(&out, "['a', 'b', 'c,d']\n['a,b', 'c', 'd']\n");
}

#[test]
fn test_str_splitlines_and_consecutive_delim() {
    let out = jit_capture(
        r#"print("a\nb\nc".splitlines())
print("hello".split("l"))
print("a,,b".split(","))
"#,
    );
    assert_output(&out, "['a', 'b', 'c']\n['he', '', 'o']\n['a', '', 'b']\n");
}
