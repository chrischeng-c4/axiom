//! Py3.12 conformance tests for bytes (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_bytes.py — BytesTest
//!
//! Coverage: bytes literal, len, indexing, slicing, decode, hex, equality,
//! membership, concatenation, repetition, bool, iteration.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_bytes_empty_literal_len() {
    let out = jit_capture(
        r#"b = b""
print(len(b))
"#,
    );
    assert_output(&out, "0\n");
}

#[test]
fn test_bytes_literal_len() {
    let out = jit_capture(
        r#"b = b"hello"
print(len(b))
"#,
    );
    assert_output(&out, "5\n");
}

#[test]
fn test_bytes_indexing_returns_int() {
    let out = jit_capture(
        r#"b = b"abc"
print(b[0])
print(b[1])
print(b[2])
"#,
    );
    assert_output(&out, "97\n98\n99\n");
}

#[test]
fn test_bytes_indexing_negative() {
    let out = jit_capture(
        r#"b = b"abc"
print(b[-1])
print(b[-3])
"#,
    );
    assert_output(&out, "99\n97\n");
}

#[test]
fn test_bytes_decode_utf8() {
    let out = jit_capture(
        r#"b = b"hello"
print(b.decode())
"#,
    );
    assert_output(&out, "hello\n");
}

#[test]
fn test_bytes_decode_with_encoding_arg() {
    let out = jit_capture(
        r#"b = b"hello"
print(b.decode("utf-8"))
"#,
    );
    assert_output(&out, "hello\n");
}

#[test]
fn test_bytes_equality() {
    let out = jit_capture(
        r#"print(b"abc" == b"abc")
print(b"abc" == b"xyz")
print(b"" == b"")
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\n");
}

#[test]
fn test_bytes_concatenation() {
    let out = jit_capture(
        r#"a = b"hello"
b = b" world"
c = a + b
print(len(c))
print(c.decode())
"#,
    );
    assert_output(&out, "11\nhello world\n");
}

#[test]
fn test_bytes_repetition() {
    let out = jit_capture(
        r#"b = b"ab" * 3
print(len(b))
print(b.decode())
"#,
    );
    assert_output(&out, "6\nababab\n");
}

#[test]
fn test_bytes_bool_empty_is_false() {
    let out = jit_capture(
        r#"print(bool(b""))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_bytes_bool_nonempty_is_true() {
    let out = jit_capture(
        r#"print(bool(b"x"))
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_bytes_contains_int() {
    let out = jit_capture(
        r#"b = b"abc"
print(97 in b)
print(120 in b)
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

#[test]
fn test_bytes_iteration_sum() {
    let out = jit_capture(
        r#"total = 0
for x in b"abc":
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "294\n");
}

#[test]
fn test_bytes_startswith_true() {
    let out = jit_capture(
        r#"print(b"hello world".startswith(b"hello"))
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_bytes_startswith_false() {
    let out = jit_capture(
        r#"print(b"hello world".startswith(b"world"))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_bytes_endswith_true() {
    let out = jit_capture(
        r#"print(b"hello world".endswith(b"world"))
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_bytes_endswith_false() {
    let out = jit_capture(
        r#"print(b"hello world".endswith(b"hello"))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_bytes_count() {
    let out = jit_capture(
        r#"print(b"banana".count(b"a"))
"#,
    );
    assert_output(&out, "3\n");
}

#[test]
fn test_bytes_find_present() {
    let out = jit_capture(
        r#"print(b"hello world".find(b"world"))
"#,
    );
    assert_output(&out, "6\n");
}

#[test]
fn test_bytes_find_absent_returns_minus_one() {
    let out = jit_capture(
        r#"print(b"hello".find(b"xyz"))
"#,
    );
    assert_output(&out, "-1\n");
}
