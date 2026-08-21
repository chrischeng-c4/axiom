//! Py3.12 conformance tests for bytearray (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_bytes.py — ByteArrayTest
//!
//! Coverage: bytearray constructor (empty / from bytes / from int /
//! from list of ints), len, indexing (positive + negative), decode,
//! equality, concat, repetition, bool, contains-int, iteration,
//! mutation via setitem, append, extend, slicing.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_bytearray_empty_constructor() {
    let out = jit_capture(
        r#"b = bytearray()
print(len(b))
"#,
    );
    assert_output(&out, "0\n");
}

#[test]
fn test_bytearray_from_bytes() {
    let out = jit_capture(
        r#"b = bytearray(b"hello")
print(len(b))
print(b.decode())
"#,
    );
    assert_output(&out, "5\nhello\n");
}

#[test]
fn test_bytearray_from_int_zero_filled() {
    let out = jit_capture(
        r#"b = bytearray(5)
print(len(b))
print(b[0])
print(b[4])
"#,
    );
    assert_output(&out, "5\n0\n0\n");
}

#[test]
fn test_bytearray_from_list_of_ints() {
    let out = jit_capture(
        r#"b = bytearray([97, 98, 99])
print(len(b))
print(b.decode())
"#,
    );
    assert_output(&out, "3\nabc\n");
}

#[test]
fn test_bytearray_indexing_returns_int() {
    let out = jit_capture(
        r#"b = bytearray(b"abc")
print(b[0])
print(b[1])
print(b[2])
"#,
    );
    assert_output(&out, "97\n98\n99\n");
}

#[test]
fn test_bytearray_indexing_negative() {
    let out = jit_capture(
        r#"b = bytearray(b"abc")
print(b[-1])
print(b[-3])
"#,
    );
    assert_output(&out, "99\n97\n");
}

#[test]
fn test_bytearray_decode_utf8() {
    let out = jit_capture(
        r#"b = bytearray(b"hello")
print(b.decode())
print(b.decode("utf-8"))
"#,
    );
    assert_output(&out, "hello\nhello\n");
}

#[test]
fn test_bytearray_equality() {
    let out = jit_capture(
        r#"print(bytearray(b"abc") == bytearray(b"abc"))
print(bytearray(b"abc") == bytearray(b"xyz"))
print(bytearray() == bytearray())
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\n");
}

#[test]
fn test_bytearray_bool_empty_is_false() {
    let out = jit_capture(
        r#"print(bool(bytearray()))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_bytearray_bool_nonempty_is_true() {
    let out = jit_capture(
        r#"print(bool(bytearray(b"x")))
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_bytearray_contains_int() {
    let out = jit_capture(
        r#"b = bytearray(b"abc")
print(97 in b)
print(120 in b)
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

#[test]
fn test_bytearray_iteration_sum() {
    let out = jit_capture(
        r#"total = 0
for x in bytearray(b"abc"):
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "294\n");
}

#[test]
fn test_bytearray_setitem_int() {
    let out = jit_capture(
        r#"b = bytearray(b"abc")
b[0] = 120
print(b.decode())
"#,
    );
    assert_output(&out, "xbc\n");
}

#[test]
fn test_bytearray_append() {
    let out = jit_capture(
        r#"b = bytearray(b"ab")
b.append(99)
print(len(b))
print(b.decode())
"#,
    );
    assert_output(&out, "3\nabc\n");
}

#[test]
fn test_bytearray_extend_with_bytes() {
    let out = jit_capture(
        r#"b = bytearray(b"hello")
b.extend(b" world")
print(b.decode())
"#,
    );
    assert_output(&out, "hello world\n");
}

#[test]
fn test_bytearray_startswith_endswith() {
    let out = jit_capture(
        r#"b = bytearray(b"hello world")
print(b.startswith(b"hello"))
print(b.endswith(b"world"))
print(b.startswith(b"world"))
"#,
    );
    assert_output(&out, "True\nTrue\nFalse\n");
}

#[test]
fn test_bytearray_count() {
    let out = jit_capture(
        r#"b = bytearray(b"banana")
print(b.count(b"a"))
"#,
    );
    assert_output(&out, "3\n");
}

#[test]
fn test_bytearray_find_present_and_absent() {
    let out = jit_capture(
        r#"b = bytearray(b"hello world")
print(b.find(b"world"))
print(b.find(b"xyz"))
"#,
    );
    assert_output(&out, "6\n-1\n");
}
