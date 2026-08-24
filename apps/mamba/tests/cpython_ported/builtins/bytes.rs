//! Ported from Lib/test/test_bytes_ported.py
//! Integration tests: builtins/bytes.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_bytes_ported.py`.
#[test]
fn test_bytes_empty_literal_len() {
    let out = jit_capture(
        r#"b = b""
print(len(b))
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_bytes_ported.py`.
#[test]
fn test_bytes_literal_len() {
    let out = jit_capture(
        r#"b = b"hello"
print(len(b))
"#,
    );
    assert_output(&out, "5\n");
}

/// Ported from `Lib/test/test_bytes_ported.py`.
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

/// Ported from `Lib/test/test_bytes_ported.py`.
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

/// Ported from `Lib/test/test_bytes_ported.py`.
#[test]
fn test_bytes_decode_utf8() {
    let out = jit_capture(
        r#"b = b"hello"
print(b.decode())
"#,
    );
    assert_output(&out, "hello\n");
}

/// Ported from `Lib/test/test_bytes_ported.py`.
#[test]
fn test_bytes_decode_with_encoding_arg() {
    let out = jit_capture(
        r#"b = b"hello"
print(b.decode("utf-8"))
"#,
    );
    assert_output(&out, "hello\n");
}

/// Ported from `Lib/test/test_bytes_ported.py`.
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

/// Ported from `Lib/test/test_bytes_ported.py`.
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

/// Ported from `Lib/test/test_bytes_ported.py`.
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

/// Ported from `Lib/test/test_bytes_ported.py`.
#[test]
fn test_bytes_bool_empty_is_false() {
    let out = jit_capture(
        r#"print(bool(b""))
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_bytes_ported.py`.
#[test]
fn test_bytes_bool_nonempty_is_true() {
    let out = jit_capture(
        r#"print(bool(b"x"))
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_bytes_ported.py`.
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

/// Ported from `Lib/test/test_bytes_ported.py`.
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

/// Ported from `Lib/test/test_bytes_ported.py`.
#[test]
fn test_bytes_startswith_true() {
    let out = jit_capture(
        r#"print(b"hello world".startswith(b"hello"))
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_bytes_ported.py`.
#[test]
fn test_bytes_startswith_false() {
    let out = jit_capture(
        r#"print(b"hello world".startswith(b"world"))
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_bytes_ported.py`.
#[test]
fn test_bytes_endswith_true() {
    let out = jit_capture(
        r#"print(b"hello world".endswith(b"world"))
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_bytes_ported.py`.
#[test]
fn test_bytes_endswith_false() {
    let out = jit_capture(
        r#"print(b"hello world".endswith(b"hello"))
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_bytes_ported.py`.
#[test]
fn test_bytes_count() {
    let out = jit_capture(
        r#"print(b"banana".count(b"a"))
"#,
    );
    assert_output(&out, "3\n");
}

/// Ported from `Lib/test/test_bytes_ported.py`.
#[test]
fn test_bytes_find_present() {
    let out = jit_capture(
        r#"print(b"hello world".find(b"world"))
"#,
    );
    assert_output(&out, "6\n");
}

/// Ported from `Lib/test/test_bytes_ported.py`.
#[test]
fn test_bytes_find_absent_returns_minus_one() {
    let out = jit_capture(
        r#"print(b"hello".find(b"xyz"))
"#,
    );
    assert_output(&out, "-1\n");
}

/// Ported from `Lib/test/test_bytes_ported.py`.
#[test]
fn test_iter_bytes_for_loop() {
    let out = jit_capture(
        r#"total = 0
for b in b"abc":
    total = total + b
print(total)
"#,
    );
    assert_output(&out, "294\n");
}

/// Ported from `Lib/test/test_bytes_ported.py`.
#[test]
fn test_bytes_decode_to_text() {
    let out = jit_capture(
        r#"print(b"hello".decode())
print(b"abc".decode())
"#,
    );
    assert_output(&out, "hello\nabc\n");
}

/// Ported from `Lib/test/test_bytes_ported.py`.
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

/// Ported from `Lib/test/test_bytes_ported.py`.
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

/// Ported from `Lib/test/test_bytes_ported.py`.
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

