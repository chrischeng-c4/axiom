//! Py3.12 conformance tests for the `base64` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_base64.py):
//!   b64encode / b64decode round-trips; urlsafe variant; b16/b32 variants.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_base64_b64encode_hello() {
    let out = jit_capture(
        r#"import base64
print(base64.b64encode(b"hello").decode())
"#,
    );
    assert_output(&out, "aGVsbG8=\n");
}

#[test]
fn test_base64_b64decode_hello() {
    let out = jit_capture(
        r#"import base64
print(base64.b64decode("aGVsbG8=").decode())
"#,
    );
    assert_output(&out, "hello\n");
}

#[test]
fn test_base64_b64_roundtrip() {
    let out = jit_capture(
        r#"import base64
data = b"the quick brown fox"
encoded = base64.b64encode(data)
decoded = base64.b64decode(encoded)
print(decoded == data)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_base64_b64encode_empty() {
    let out = jit_capture(
        r#"import base64
print(base64.b64encode(b"").decode())
"#,
    );
    assert_output(&out, "\n");
}

#[test]
fn test_base64_urlsafe_roundtrip() {
    let out = jit_capture(
        r#"import base64
data = b"\xfb\xff\xff"
encoded = base64.urlsafe_b64encode(data)
print(encoded.decode())
print(base64.urlsafe_b64decode(encoded) == data)
"#,
    );
    assert_output(&out, "-___\nTrue\n");
}

#[test]
fn test_base64_b16encode() {
    let out = jit_capture(
        r#"import base64
print(base64.b16encode(b"hi").decode())
print(base64.b16decode("6869").decode())
"#,
    );
    assert_output(&out, "6869\nhi\n");
}
