//! Py3.12 conformance tests for common `str` methods (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unicode.py and
//! string_tests.py — basic method sections):
//!   `replace`, `split`/`join`, `find`/`count`, `startswith`/`endswith`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_str_replace_substitutes_substring() {
    let out = jit_capture(
        r#"print("hello world".replace("world", "Python"))
print("aaa".replace("a", "b"))
print("xxx".replace("y", "z"))
"#,
    );
    assert_output(&out, "hello Python\nbbb\nxxx\n");
}

#[test]
fn test_str_split_and_join_roundtrip() {
    let out = jit_capture(
        r#"parts = "a,b,c".split(",")
print(parts)
print("-".join(parts))
print(",".join(["x", "y", "z"]))
"#,
    );
    assert_output(&out, "['a', 'b', 'c']\na-b-c\nx,y,z\n");
}

#[test]
fn test_str_find_and_count() {
    let out = jit_capture(
        r#"print("hello".find("l"))
print("hello".find("z"))
print("banana".count("a"))
print("banana".count("na"))
"#,
    );
    assert_output(&out, "2\n-1\n3\n2\n");
}

#[test]
fn test_str_startswith_and_endswith() {
    let out = jit_capture(
        r#"print("hello".startswith("he"))
print("hello".startswith("lo"))
print("hello".endswith("lo"))
print("hello".endswith("he"))
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nFalse\n");
}
