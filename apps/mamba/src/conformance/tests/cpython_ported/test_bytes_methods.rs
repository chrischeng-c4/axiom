//! Py3.12 conformance tests for `bytes` accessor methods (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_bytes.py — accessor
//! sections):
//!   `decode` to text, `startswith`/`endswith`, `find`/`count`,
//!   concatenation and `len`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_bytes_decode_to_text() {
    let out = jit_capture(
        r#"print(b"hello".decode())
print(b"abc".decode())
"#,
    );
    assert_output(&out, "hello\nabc\n");
}

#[test]
fn test_bytes_startswith_endswith() {
    let out = jit_capture(
        r#"print(b"hello".startswith(b"he"))
print(b"hello".startswith(b"lo"))
print(b"hello".endswith(b"lo"))
print(b"hello".endswith(b"he"))
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nFalse\n");
}

#[test]
fn test_bytes_find_and_count() {
    let out = jit_capture(
        r#"print(b"hello".find(b"l"))
print(b"hello".find(b"z"))
print(b"hello".count(b"l"))
print(b"banana".count(b"a"))
"#,
    );
    assert_output(&out, "2\n-1\n2\n3\n");
}

#[test]
fn test_bytes_concat_and_len() {
    let out = jit_capture(
        r#"print(b"a" + b"bc")
print(len(b"hello"))
print(len(b""))
print(b"x" * 3)
"#,
    );
    assert_output(&out, "b'abc'\n5\n0\nb'xxx'\n");
}
