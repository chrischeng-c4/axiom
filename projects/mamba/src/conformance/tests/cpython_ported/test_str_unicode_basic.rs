//! Py3.12 conformance tests for basic Unicode handling in `str`
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unicode.py — basic
//! Unicode sections):
//!   non-ASCII char literal print, `ord`/`chr` for non-ASCII
//!   codepoints, `len` counting characters (not bytes), and `[]`
//!   character access including negative index.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_str_prints_non_ascii_characters() {
    let out = jit_capture(
        r#"print("é")
print("ñ")
print("ü")
"#,
    );
    assert_output(&out, "é\nñ\nü\n");
}

#[test]
fn test_str_ord_chr_on_non_ascii_codepoints() {
    let out = jit_capture(
        r#"print(ord("é"))
print(chr(233))
print(ord(chr(255)))
"#,
    );
    assert_output(&out, "233\né\n255\n");
}

#[test]
fn test_str_len_counts_characters_not_bytes() {
    let out = jit_capture(
        r#"print(len("café"))
print(len("año"))
print(len("ascii"))
"#,
    );
    assert_output(&out, "4\n3\n5\n");
}

#[test]
fn test_str_indexing_first_and_last() {
    let out = jit_capture(
        r#"print("hello"[0])
print("hello"[-1])
print("hello"[2])
"#,
    );
    assert_output(&out, "h\no\nl\n");
}
