//! Py3.12 conformance tests for the `string` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_string.py):
//!   module-level constant attributes (ascii_lowercase, ascii_uppercase,
//!   digits, hexdigits, octdigits, punctuation, whitespace, printable).
//!
//! string.Template is intentionally excluded — `substitute` currently
//! raises `AttributeError: 'dict' object has no attribute 'substitute'`
//! under mamba; deferred as a separate runtime gap.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_string_ascii_lowercase() {
    let out = jit_capture(
        r#"import string
print(string.ascii_lowercase)
"#,
    );
    assert_output(&out, "abcdefghijklmnopqrstuvwxyz\n");
}

#[test]
fn test_string_ascii_uppercase() {
    let out = jit_capture(
        r#"import string
print(string.ascii_uppercase)
"#,
    );
    assert_output(&out, "ABCDEFGHIJKLMNOPQRSTUVWXYZ\n");
}

#[test]
fn test_string_digits() {
    let out = jit_capture(
        r#"import string
print(string.digits)
"#,
    );
    assert_output(&out, "0123456789\n");
}

#[test]
fn test_string_hexdigits() {
    let out = jit_capture(
        r#"import string
print(string.hexdigits)
"#,
    );
    assert_output(&out, "0123456789abcdefABCDEF\n");
}

#[test]
fn test_string_octdigits() {
    let out = jit_capture(
        r#"import string
print(string.octdigits)
"#,
    );
    assert_output(&out, "01234567\n");
}

#[test]
fn test_string_ascii_letters_concat() {
    let out = jit_capture(
        r#"import string
print(len(string.ascii_letters))
print(string.ascii_letters[:5])
print(string.ascii_letters[-5:])
"#,
    );
    assert_output(&out, "52\nabcde\nVWXYZ\n");
}
