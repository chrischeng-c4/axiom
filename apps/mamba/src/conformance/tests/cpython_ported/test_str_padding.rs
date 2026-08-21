//! Py3.12 conformance tests for string padding/justification (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unicode.py — `ljust`,
//! `rjust`, `center`, `zfill` sections):
//!   default-space pad, custom fill char, zfill on negative numbers.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_str_default_space_padding() {
    let out = jit_capture(
        r#"print(repr("hi".ljust(5)))
print(repr("hi".rjust(5)))
print(repr("hi".center(5)))
"#,
    );
    assert_output(&out, "'hi   '\n'   hi'\n'  hi '\n");
}

#[test]
fn test_str_custom_fill_character() {
    let out = jit_capture(
        r#"print("hi".ljust(5, "*"))
print("hi".rjust(5, "*"))
print("hi".center(5, "*"))
"#,
    );
    assert_output(&out, "hi***\n***hi\n**hi*\n");
}

#[test]
fn test_str_zfill_preserves_sign() {
    let out = jit_capture(
        r#"print("42".zfill(5))
print("-7".zfill(5))
print("0".zfill(3))
"#,
    );
    assert_output(&out, "00042\n-0007\n000\n");
}
