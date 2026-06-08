//! Py3.12 conformance tests for the `secrets` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_secrets.py):
//!   token_hex length, randbelow range, choice element membership.
//!   Property tests only — values are unpredictable by design.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_secrets_token_hex_length() {
    let out = jit_capture(
        r#"import secrets
print(len(secrets.token_hex(8)))
print(len(secrets.token_hex(16)))
"#,
    );
    assert_output(&out, "16\n32\n");
}

#[test]
fn test_secrets_randbelow_range() {
    let out = jit_capture(
        r#"import secrets
v = secrets.randbelow(100)
print(0 <= v < 100)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_secrets_choice_from_population() {
    let out = jit_capture(
        r#"import secrets
v = secrets.choice([1, 2, 3, 4, 5])
print(v in [1, 2, 3, 4, 5])
"#,
    );
    assert_output(&out, "True\n");
}
