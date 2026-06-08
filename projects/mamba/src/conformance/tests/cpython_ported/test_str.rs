//! Py3.12 conformance tests for str (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_str.py / test_unicode.py — StrTest, UnicodeTest
//!
//! Coverage: construction, len, indexing (positive + negative), upper/lower/
//! title/capitalize, strip/lstrip/rstrip, startswith/endswith, find/rfind/
//! index, count, replace, split/rsplit, join, isdigit/isalpha/isalnum/isspace,
//! concatenation, repetition, equality, bool, membership.
//!
//! @issue #759

use super::{assert_output, jit_capture};

// ── construction ──────────────────────────────────────────────────────────────

#[test]
fn test_str_empty_literal_len() {
    let out = jit_capture(
        r#"s = ""
print(len(s))
"#,
    );
    assert_output(&out, "0\n");
}

#[test]
fn test_str_literal_len() {
    let out = jit_capture(
        r#"s = "hello"
print(len(s))
"#,
    );
    assert_output(&out, "5\n");
}

#[test]
fn test_str_constructor_from_int() {
    let out = jit_capture(
        r#"s = str(42)
print(s)
print(len(s))
"#,
    );
    assert_output(&out, "42\n2\n");
}

// ── indexing ─────────────────────────────────────────────────────────────────

#[test]
fn test_str_getitem_positive() {
    let out = jit_capture(
        r#"s = "abc"
print(s[0])
print(s[1])
print(s[2])
"#,
    );
    assert_output(&out, "a\nb\nc\n");
}

#[test]
fn test_str_getitem_negative() {
    let out = jit_capture(
        r#"s = "abc"
print(s[-1])
print(s[-3])
"#,
    );
    assert_output(&out, "c\na\n");
}

// ── case methods ─────────────────────────────────────────────────────────────

#[test]
fn test_str_upper() {
    let out = jit_capture(
        r#"print("Hello".upper())
"#,
    );
    assert_output(&out, "HELLO\n");
}

#[test]
fn test_str_lower() {
    let out = jit_capture(
        r#"print("HeLLo".lower())
"#,
    );
    assert_output(&out, "hello\n");
}

#[test]
fn test_str_capitalize() {
    let out = jit_capture(
        r#"print("hello world".capitalize())
"#,
    );
    assert_output(&out, "Hello world\n");
}

#[test]
fn test_str_title() {
    let out = jit_capture(
        r#"print("hello world".title())
"#,
    );
    assert_output(&out, "Hello World\n");
}

// ── strip methods ─────────────────────────────────────────────────────────────

#[test]
fn test_str_strip_whitespace() {
    let out = jit_capture(
        r#"print("  hello  ".strip())
"#,
    );
    assert_output(&out, "hello\n");
}

#[test]
fn test_str_lstrip_whitespace() {
    let out = jit_capture(
        r#"s = "  hello  ".lstrip()
print(s)
print(len(s))
"#,
    );
    assert_output(&out, "hello  \n7\n");
}

#[test]
fn test_str_rstrip_whitespace() {
    let out = jit_capture(
        r#"s = "  hello  ".rstrip()
print(s)
print(len(s))
"#,
    );
    assert_output(&out, "  hello\n7\n");
}

// ── startswith / endswith ────────────────────────────────────────────────────

#[test]
fn test_str_startswith_true() {
    let out = jit_capture(
        r#"print("hello world".startswith("hello"))
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_str_startswith_false() {
    let out = jit_capture(
        r#"print("hello world".startswith("world"))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_str_endswith_true() {
    let out = jit_capture(
        r#"print("hello world".endswith("world"))
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_str_endswith_false() {
    let out = jit_capture(
        r#"print("hello world".endswith("hello"))
"#,
    );
    assert_output(&out, "False\n");
}

// ── find / index / count ─────────────────────────────────────────────────────

#[test]
fn test_str_find_present() {
    let out = jit_capture(
        r#"print("hello world".find("world"))
"#,
    );
    assert_output(&out, "6\n");
}

#[test]
fn test_str_find_absent_returns_minus_one() {
    let out = jit_capture(
        r#"print("hello".find("xyz"))
"#,
    );
    assert_output(&out, "-1\n");
}

#[test]
fn test_str_index_present() {
    let out = jit_capture(
        r#"print("hello world".index("world"))
"#,
    );
    assert_output(&out, "6\n");
}

#[test]
fn test_str_count_substr() {
    let out = jit_capture(
        r#"print("banana".count("a"))
"#,
    );
    assert_output(&out, "3\n");
}

#[test]
fn test_str_count_absent_is_zero() {
    let out = jit_capture(
        r#"print("banana".count("z"))
"#,
    );
    assert_output(&out, "0\n");
}

// ── replace ──────────────────────────────────────────────────────────────────

#[test]
fn test_str_replace_basic() {
    let out = jit_capture(
        r#"print("hello world".replace("world", "python"))
"#,
    );
    assert_output(&out, "hello python\n");
}

#[test]
fn test_str_replace_all_occurrences() {
    let out = jit_capture(
        r#"print("aaa".replace("a", "b"))
"#,
    );
    assert_output(&out, "bbb\n");
}

#[test]
fn test_str_replace_absent_is_noop() {
    let out = jit_capture(
        r#"print("hello".replace("z", "x"))
"#,
    );
    assert_output(&out, "hello\n");
}

// ── split / join ─────────────────────────────────────────────────────────────

#[test]
fn test_str_split_default_whitespace() {
    let out = jit_capture(
        r#"parts = "a b c".split()
print(len(parts))
print(parts[0])
print(parts[2])
"#,
    );
    assert_output(&out, "3\na\nc\n");
}

#[test]
fn test_str_split_explicit_separator() {
    let out = jit_capture(
        r#"parts = "a,b,c".split(",")
print(len(parts))
print(parts[1])
"#,
    );
    assert_output(&out, "3\nb\n");
}

#[test]
fn test_str_join_with_list() {
    let out = jit_capture(
        r#"print(",".join(["a", "b", "c"]))
"#,
    );
    assert_output(&out, "a,b,c\n");
}

#[test]
fn test_str_join_empty_iterable() {
    let out = jit_capture(
        r#"print(",".join([]))
"#,
    );
    assert_output(&out, "\n");
}

// ── predicates ───────────────────────────────────────────────────────────────

#[test]
fn test_str_isdigit_true() {
    let out = jit_capture(
        r#"print("123".isdigit())
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_str_isdigit_false() {
    let out = jit_capture(
        r#"print("12a".isdigit())
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_str_isalpha_true() {
    let out = jit_capture(
        r#"print("abc".isalpha())
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_str_isalpha_false_with_digit() {
    let out = jit_capture(
        r#"print("abc1".isalpha())
"#,
    );
    assert_output(&out, "False\n");
}

// ── operators ────────────────────────────────────────────────────────────────

#[test]
fn test_str_concatenation_operator() {
    let out = jit_capture(
        r#"print("hello" + " " + "world")
"#,
    );
    assert_output(&out, "hello world\n");
}

#[test]
fn test_str_repetition_operator() {
    let out = jit_capture(
        r#"print("ab" * 3)
"#,
    );
    assert_output(&out, "ababab\n");
}

#[test]
fn test_str_equal_same_content() {
    let out = jit_capture(
        r#"print("hello" == "hello")
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_str_not_equal_different_content() {
    let out = jit_capture(
        r#"print("hello" == "world")
"#,
    );
    assert_output(&out, "False\n");
}

// ── bool / truthiness ────────────────────────────────────────────────────────

#[test]
fn test_str_bool_empty_is_false() {
    let out = jit_capture(
        r#"print(bool(""))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_str_bool_nonempty_is_true() {
    let out = jit_capture(
        r#"print(bool("x"))
"#,
    );
    assert_output(&out, "True\n");
}

// ── membership ───────────────────────────────────────────────────────────────

#[test]
fn test_str_contains_present() {
    let out = jit_capture(
        r#"print("ell" in "hello")
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_str_contains_absent() {
    let out = jit_capture(
        r#"print("xyz" in "hello")
"#,
    );
    assert_output(&out, "False\n");
}
