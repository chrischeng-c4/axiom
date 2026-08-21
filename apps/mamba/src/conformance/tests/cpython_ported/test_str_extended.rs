//! Py3.12 conformance tests for str — extended coverage (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unicode.py and test_str):
//!   formatting helpers (center/ljust/rjust/zfill), case predicates
//!   (isalnum/isspace/isupper/islower/istitle), positional finders
//!   (rfind, find with chars-set strip), slicing, repr.
//!
//! Complements `test_str.rs` (basic surface).
//!
//! @issue #759

use super::{assert_output, jit_capture};

// ---------------------------------------------------------------- formatters

#[test]
fn test_str_center_with_fill() {
    let out = jit_capture(r#"print("abc".center(7, "*"))"#);
    assert_output(&out, "**abc**\n");
}

#[test]
fn test_str_ljust_with_fill() {
    let out = jit_capture(r#"print("abc".ljust(7, "-"))"#);
    assert_output(&out, "abc----\n");
}

#[test]
fn test_str_rjust_with_fill() {
    let out = jit_capture(r#"print("abc".rjust(7, "-"))"#);
    assert_output(&out, "----abc\n");
}

#[test]
fn test_str_zfill() {
    let out = jit_capture(r#"print("42".zfill(5))"#);
    assert_output(&out, "00042\n");
}

// ---------------------------------------------------------------- find variants

#[test]
fn test_str_rfind_present() {
    let out = jit_capture(r#"print("hello".rfind("l"))"#);
    assert_output(&out, "3\n");
}

#[test]
fn test_str_rfind_absent() {
    let out = jit_capture(r#"print("hello".rfind("z"))"#);
    assert_output(&out, "-1\n");
}

// ---------------------------------------------------------------- strip with chars

#[test]
fn test_str_strip_chars() {
    let out = jit_capture(r#"print("xxhelloxx".strip("x"))"#);
    assert_output(&out, "hello\n");
}

#[test]
fn test_str_lstrip_chars() {
    let out = jit_capture(r#"print("xxhello".lstrip("x"))"#);
    assert_output(&out, "hello\n");
}

#[test]
fn test_str_rstrip_chars() {
    let out = jit_capture(r#"print("helloxx".rstrip("x"))"#);
    assert_output(&out, "hello\n");
}

// ---------------------------------------------------------------- case predicates

#[test]
fn test_str_isalnum_true() {
    let out = jit_capture(r#"print("abc123".isalnum())"#);
    assert_output(&out, "True\n");
}

#[test]
fn test_str_isalnum_false_with_space() {
    let out = jit_capture(r#"print("abc 123".isalnum())"#);
    assert_output(&out, "False\n");
}

#[test]
fn test_str_isspace_true() {
    let out = jit_capture(r#"print("   ".isspace())"#);
    assert_output(&out, "True\n");
}

#[test]
fn test_str_isspace_false() {
    let out = jit_capture(r#"print("a b".isspace())"#);
    assert_output(&out, "False\n");
}

#[test]
fn test_str_isupper_true() {
    let out = jit_capture(r#"print("ABC".isupper())"#);
    assert_output(&out, "True\n");
}

#[test]
fn test_str_isupper_false() {
    let out = jit_capture(r#"print("Abc".isupper())"#);
    assert_output(&out, "False\n");
}

#[test]
fn test_str_islower_true() {
    let out = jit_capture(r#"print("abc".islower())"#);
    assert_output(&out, "True\n");
}

#[test]
fn test_str_islower_false() {
    let out = jit_capture(r#"print("Abc".islower())"#);
    assert_output(&out, "False\n");
}

#[test]
fn test_str_istitle_true() {
    let out = jit_capture(r#"print("Title Case".istitle())"#);
    assert_output(&out, "True\n");
}

#[test]
fn test_str_istitle_false() {
    let out = jit_capture(r#"print("not title".istitle())"#);
    assert_output(&out, "False\n");
}

// ---------------------------------------------------------------- slicing

#[test]
fn test_str_slice_range() {
    let out = jit_capture(r#"print("hello"[1:4])"#);
    assert_output(&out, "ell\n");
}

#[test]
fn test_str_slice_reverse() {
    let out = jit_capture(r#"print("hello"[::-1])"#);
    assert_output(&out, "olleh\n");
}

#[test]
fn test_str_slice_step() {
    let out = jit_capture(r#"print("abcdef"[::2])"#);
    assert_output(&out, "ace\n");
}

#[test]
fn test_str_slice_to_end() {
    let out = jit_capture(r#"print("hello"[2:])"#);
    assert_output(&out, "llo\n");
}

#[test]
fn test_str_slice_from_start() {
    let out = jit_capture(r#"print("hello"[:3])"#);
    assert_output(&out, "hel\n");
}

// ---------------------------------------------------------------- repr / encode

#[test]
fn test_str_repr() {
    let out = jit_capture(r#"print(repr("abc"))"#);
    assert_output(&out, "'abc'\n");
}

#[test]
fn test_str_encode_utf8() {
    let out = jit_capture(r#"print("hello".encode("utf-8"))"#);
    assert_output(&out, "b'hello'\n");
}

// ---------------------------------------------------------------- replace nuances

#[test]
fn test_str_replace_with_count() {
    let out = jit_capture(r#"print("aaaa".replace("a", "b", 2))"#);
    assert_output(&out, "bbaa\n");
}

// ---------------------------------------------------------------- multi-char split / partition-like

#[test]
fn test_str_split_max_splits() {
    let out = jit_capture(r#"print("a,b,c,d".split(",", 2))"#);
    assert_output(&out, "['a', 'b', 'c,d']\n");
}

#[test]
fn test_str_rsplit_max_splits() {
    let out = jit_capture(r#"print("a,b,c,d".rsplit(",", 2))"#);
    assert_output(&out, "['a,b', 'c', 'd']\n");
}

#[test]
fn test_str_splitlines() {
    let out = jit_capture(r#"print("a\nb\nc".splitlines())"#);
    assert_output(&out, "['a', 'b', 'c']\n");
}
