//! Py3.12 conformance tests for the `unicodedata` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unicodedata.py):
//!   category, normalize. `unicodedata.name` is intentionally excluded —
//!   mamba currently returns synthesized "UNICODE CHAR XXXX" labels
//!   instead of the canonical names; deferred as a separate gap.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_unicodedata_category_uppercase_letter() {
    let out = jit_capture(
        r#"import unicodedata
print(unicodedata.category("A"))
"#,
    );
    assert_output(&out, "Lu\n");
}

#[test]
fn test_unicodedata_category_lowercase_letter() {
    let out = jit_capture(
        r#"import unicodedata
print(unicodedata.category("a"))
"#,
    );
    assert_output(&out, "Ll\n");
}

#[test]
fn test_unicodedata_category_decimal_digit() {
    let out = jit_capture(
        r#"import unicodedata
print(unicodedata.category("5"))
"#,
    );
    assert_output(&out, "Nd\n");
}

#[test]
fn test_unicodedata_normalize_nfc_identity_ascii() {
    let out = jit_capture(
        r#"import unicodedata
print(unicodedata.normalize("NFC", "hello"))
"#,
    );
    assert_output(&out, "hello\n");
}
